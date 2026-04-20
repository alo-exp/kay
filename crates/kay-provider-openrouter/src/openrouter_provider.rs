//! `OpenRouterProvider` — concrete `Provider` impl (PROV-01, D-02).
//!
//! Composes:
//!   - `Allowlist`   (plan 02-07) — pre-HTTP gate + wire-model rewrite.
//!   - `ApiKey`      (plan 02-07) — redacted credential with env/config
//!     precedence resolution.
//!   - `UpstreamClient` (plan 02-08 T1) — reqwest POST + SSE handshake.
//!   - `translate_stream` (plan 02-08 T2) — per-tool_call.id reassembly.
//!
//! Pre-flight order (per `Provider::chat`):
//!   1. `allowlist.check(&request.model)?` — rejects non-allowlisted IDs
//!      BEFORE any HTTP call (TM-04, Pitfall 8).
//!   2. Wire-model rewrite via `to_wire_model` (appends `:exacto`, TM-08).
//!   3. `build_request_body` → JSON bytes (stream=true).
//!   4. `upstream.stream_chat(body)` → `EventSource`.
//!   5. `translate_stream(es)` → `AgentEventStream`.
//!
//! `cost_cap: Arc<CostCap>` is pre-wired (checker BLOCKER #5 resolution)
//! with an uncapped default. Plan 02-10 T2 adds the `.max_usd()` setter
//! and the pre-flight `check()?` gate — the struct shape is stable.
//!
//! Body-construction approach: Option B (hand-rolled `serde_json`) per the
//! plan's <interfaces> allowance. The forge_app `Request` DTO is heavily
//! field-ful and requires a `ModelId` newtype + a `ToolCatalog` for tool
//! mapping — more plumbing than this wave warrants. The wire surface is
//! OpenAI-compatible; plan 02-11+ may revisit Option A once the agent loop
//! needs the transformer pipeline. **NN-7 ordering is enforced locally** —
//! serde_json is compiled with `preserve_order`, and `build_request_body`
//! inserts `required` BEFORE `properties` for every tool's parameters
//! schema. The `nn7` test asserts this on every PR.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use indexmap::IndexMap;
use reqwest::Url;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::Value;

use crate::allowlist::Allowlist;
use crate::auth::{ApiKey, ConfigAuthSource, resolve_api_key};
use crate::client::UpstreamClient;
use crate::cost_cap::CostCap;
use crate::error::ProviderError;
use crate::provider::{AgentEventStream, ChatRequest, Provider};
use crate::translator::translate_stream;

pub struct OpenRouterProvider {
    allowlist: Allowlist,
    upstream: UpstreamClient,
    /// Pre-wired per checker BLOCKER #5 (revision 2026-04-20). Uncapped
    /// default in 02-08; plan 02-10 T2 wires `.max_usd(n)` + pre-flight
    /// `check()?` through this field (no struct-shape change needed).
    cost_cap: Arc<CostCap>,
}

impl OpenRouterProvider {
    pub fn builder() -> OpenRouterProviderBuilder {
        OpenRouterProviderBuilder::default()
    }

    /// Read-only accessor for tests + diagnostics. Stable API from 02-08.
    pub fn cost_cap(&self) -> &Arc<CostCap> {
        &self.cost_cap
    }
}

#[derive(Default)]
pub struct OpenRouterProviderBuilder {
    endpoint: Option<String>,
    api_key: Option<ApiKey>,
    config_auth: Option<ConfigAuthSource>,
    allowlist: Option<Allowlist>,
    referer: Option<String>,
    title: Option<String>,
    /// Pre-introduced per checker BLOCKER #5. Plan 02-10 T2 adds the public
    /// `.max_usd(f64)` setter; today this stays `None` (uncapped default).
    max_usd: Option<f64>,
}

impl OpenRouterProviderBuilder {
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Directly supply an API key (for tests / programmatic callers).
    /// Production code should prefer `config_auth` + `OPENROUTER_API_KEY`
    /// env var for D-08 precedence.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(ApiKey::from(key.into()));
        self
    }

    pub fn config_auth(mut self, cfg: ConfigAuthSource) -> Self {
        self.config_auth = Some(cfg);
        self
    }

    pub fn allowlist(mut self, a: Allowlist) -> Self {
        self.allowlist = Some(a);
        self
    }

    pub fn referer(mut self, r: impl Into<String>) -> Self {
        self.referer = Some(r.into());
        self
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn build(self) -> Result<OpenRouterProvider, ProviderError> {
        let endpoint = self
            .endpoint
            .ok_or_else(|| ProviderError::Stream("endpoint required".into()))?;
        let endpoint_url = Url::parse(&endpoint)
            .map_err(|e| ProviderError::Stream(format!("bad endpoint url: {e}")))?;
        let allowlist = self
            .allowlist
            .unwrap_or_else(|| Allowlist::from_models(Vec::new()));
        let api_key = match self.api_key {
            Some(k) => k,
            None => resolve_api_key(self.config_auth.as_ref())?,
        };
        let upstream =
            UpstreamClient::try_new(endpoint_url, api_key)?.with_headers(self.referer, self.title);
        let cost_cap = match self.max_usd {
            None => Arc::new(CostCap::uncapped()),
            Some(n) => Arc::new(CostCap::with_cap(n)?),
        };
        Ok(OpenRouterProvider {
            allowlist,
            upstream,
            cost_cap,
        })
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError> {
        // 1. Pre-flight allowlist gate (TM-04, Pitfall 8). Rejection never
        //    sends a byte to OpenRouter.
        self.allowlist.check(&request.model)?;

        // 2. Wire-model rewrite — always `{canonical}:exacto` (TM-08).
        let wire_model = self.allowlist.to_wire_model(&request.model);

        // 3. Build request body (Option B: hand-rolled serde_json with
        //    NN-7 required-before-properties ordering).
        let body = build_request_body(&wire_model, &request)?;

        // 4. POST + SSE
        let es = self.upstream.stream_chat(Bytes::from(body)).await?;

        // 5. SSE → AgentEvent
        let stream = translate_stream(es);
        Ok(Box::pin(stream))
    }

    async fn models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(self.allowlist.models().to_vec())
    }
}

/// Hand-rolled request body (Option B per plan <interfaces>).
///
/// Serializes the OpenAI-compatible `POST /chat/completions` wire format
/// with `stream: true`. Tool schemas are rewritten so every `parameters`
/// object that contains BOTH `required` and `properties` emits them in the
/// order `required` then `properties` (CLAUDE.md NN-7, load-bearing for
/// Terminal-Bench 2.0 score).
///
/// NN-7 enforcement uses `indexmap::IndexMap` + a custom `Serialize` impl
/// (`OrderedObject`) that iterates keys in insertion order. This keeps
/// `serde_json/preserve_order` OFF at the workspace level — enabling it
/// would flip upstream `forge_app::dto::openai::error::Error` across the
/// `clippy::large_enum_variant` threshold (IndexMap is larger than BTreeMap),
/// and we MUST NOT patch upstream forge_code code per parity. This is the
/// Path-A variant of plan 02-08 T2 Step 6 — post-serialize reorder done
/// via a custom Serialize wrapper rather than a byte-level swap.
fn build_request_body(wire_model: &str, req: &ChatRequest) -> Result<Vec<u8>, ProviderError> {
    let messages: Vec<OrderedObject> = req
        .messages
        .iter()
        .map(|m| {
            let mut obj = OrderedObject::new();
            obj.insert("role", Value::String(m.role.clone()));
            obj.insert("content", Value::String(m.content.clone()));
            if let Some(tcid) = &m.tool_call_id {
                obj.insert("tool_call_id", Value::String(tcid.clone()));
            }
            obj
        })
        .collect();

    let tools: Vec<OrderedObject> = req.tools.iter().map(tool_to_ordered_object).collect();

    let mut body = OrderedObject::new();
    body.insert("model", Value::String(wire_model.to_string()));
    body.insert("stream", Value::Bool(true));
    body.insert_object_array("messages", messages);
    if !tools.is_empty() {
        body.insert_object_array("tools", tools);
    }
    if let Some(t) = req.temperature
        && let Some(n) = serde_json::Number::from_f64(t as f64)
    {
        body.insert("temperature", Value::Number(n));
    }
    if let Some(mx) = req.max_tokens {
        body.insert("max_tokens", Value::Number(mx.into()));
    }

    serde_json::to_vec(&body).map_err(ProviderError::Serialization)
}

/// Build a `Tool` wire object with NN-7 ordering applied to its `parameters`.
fn tool_to_ordered_object(t: &crate::provider::ToolSchema) -> OrderedObject {
    let mut function = OrderedObject::new();
    function.insert("name", Value::String(t.name.clone()));
    function.insert(
        "description",
        Value::String(t.description.clone()),
    );
    function.insert_object(
        "parameters",
        reorder_tool_parameters(&t.input_schema),
    );

    let mut tool = OrderedObject::new();
    tool.insert("type", Value::String("function".to_string()));
    tool.insert_object("function", function);
    tool
}

/// Clone a tool `parameters` schema into an `OrderedObject`. When both
/// `required` and `properties` are present, emit them in NN-7 order first.
/// All other keys keep their original (input) order. If the input is not an
/// object (e.g. a schema shorthand), it is returned verbatim inside a
/// single-field object with key `__value__` — defensive; callers should
/// pass JSON-schema-shaped objects.
fn reorder_tool_parameters(schema: &Value) -> OrderedObject {
    let mut out = OrderedObject::new();
    let Value::Object(src) = schema else {
        out.insert("__value__", schema.clone());
        return out;
    };
    let has_required = src.contains_key("required");
    let has_properties = src.contains_key("properties");

    if has_required && has_properties {
        // NN-7: required before properties
        if let Some(r) = src.get("required") {
            out.insert("required", r.clone());
        }
        if let Some(p) = src.get("properties") {
            out.insert("properties", p.clone());
        }
        for (k, v) in src {
            if k == "required" || k == "properties" {
                continue;
            }
            out.insert(k, v.clone());
        }
    } else {
        for (k, v) in src {
            out.insert(k, v.clone());
        }
    }
    out
}

/// Insertion-order-preserving JSON object backed by `IndexMap`. Keeps us
/// off of `serde_json/preserve_order` (which would leak into every
/// transitive dep and trip a clippy `large_enum_variant` threshold in
/// upstream `forge_app`). Only used inside `build_request_body` for the
/// tool-schema NN-7 ordering path; everything else is plain
/// `serde_json::Value`.
///
/// The backing stores `OVal` (Value OR nested OrderedObject OR array of
/// OVal) because `serde_json::to_value()` on an `OrderedObject` collapses
/// it back into `Value::Object(Map<...>)` which is BTreeMap-backed without
/// the `preserve_order` feature (alphabetical) — defeating NN-7. Storing a
/// nested OrderedObject as-is keeps insertion-order serialization all the
/// way down.
#[derive(Debug)]
enum OVal {
    Value(Value),
    Object(OrderedObject),
    Array(Vec<OVal>),
}

impl Serialize for OVal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            OVal::Value(v) => v.serialize(serializer),
            OVal::Object(o) => o.serialize(serializer),
            OVal::Array(arr) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

#[derive(Debug, Default)]
struct OrderedObject(IndexMap<String, OVal>);

impl OrderedObject {
    fn new() -> Self {
        Self(IndexMap::new())
    }

    fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), OVal::Value(value));
    }

    fn insert_object(&mut self, key: impl Into<String>, value: OrderedObject) {
        // Store the nested OrderedObject WITHOUT round-tripping through Value
        // (which would collapse insertion order via Map<String, Value>).
        self.0.insert(key.into(), OVal::Object(value));
    }

    fn insert_object_array(&mut self, key: impl Into<String>, items: Vec<OrderedObject>) {
        let arr = items.into_iter().map(OVal::Object).collect();
        self.0.insert(key.into(), OVal::Array(arr));
    }
}

impl Serialize for OrderedObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit {
    use super::*;

    #[test]
    fn build_request_body_emits_stream_true_and_model() {
        let req = ChatRequest {
            model: "anthropic/claude-sonnet-4.6".into(),
            messages: vec![crate::provider::Message {
                role: "user".into(),
                content: "hi".into(),
                tool_call_id: None,
            }],
            tools: vec![],
            temperature: None,
            max_tokens: None,
        };
        let bytes = build_request_body("anthropic/claude-sonnet-4.6:exacto", &req).unwrap();
        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.contains("\"model\":\"anthropic/claude-sonnet-4.6:exacto\""));
        assert!(s.contains("\"stream\":true"));
        assert!(s.contains("\"role\":\"user\""));
    }

    #[test]
    fn reorder_tool_parameters_no_required_is_identity() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "x": { "type": "string" } },
        });
        let out = reorder_tool_parameters(&schema);
        let out_value = serde_json::to_value(&out).unwrap();
        assert_eq!(out_value, schema);
    }

    #[test]
    fn reorder_tool_parameters_preserves_other_keys() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "x": { "type": "string" } },
            "required": ["x"],
            "additionalProperties": false,
        });
        let out = reorder_tool_parameters(&schema);
        let rendered = serde_json::to_string(&out).unwrap();
        // required before properties AND additionalProperties still present
        let r = rendered.find("\"required\"").unwrap();
        let p = rendered.find("\"properties\"").unwrap();
        assert!(r < p);
        assert!(rendered.contains("\"additionalProperties\":false"));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod nn7 {
    //! CLAUDE.md Non-Negotiable #7: serialized tool schemas must place
    //! `"required"` before `"properties"`. Load-bearing for TB 2.0 score.
    //! This test asserts the invariant on the `build_request_body` output.

    use super::*;
    use crate::provider::{ChatRequest, Message, ToolSchema};

    #[test]
    fn serialized_tool_schema_puts_required_before_properties() {
        let req = ChatRequest {
            model: "anthropic/claude-sonnet-4.6".into(),
            messages: vec![Message {
                role: "user".into(),
                content: "ls".into(),
                tool_call_id: None,
            }],
            tools: vec![ToolSchema {
                name: "fake_tool".into(),
                description: "Test tool for NN-7 ordering check".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["x"],
                    "properties": { "x": { "type": "string" } },
                }),
            }],
            temperature: None,
            max_tokens: None,
        };

        let body_bytes = build_request_body(&req.model, &req)
            .expect("build_request_body must succeed with a well-formed tool schema");
        let body_str = std::str::from_utf8(&body_bytes).expect("body must be valid UTF-8");

        let r_pos = body_str
            .find("\"required\"")
            .expect("NN-7 violation: serialized body missing \"required\" field");
        let p_pos = body_str
            .find("\"properties\"")
            .expect("NN-7 violation: serialized body missing \"properties\" field");
        assert!(
            r_pos < p_pos,
            "NN-7 violation: \"required\" (pos {r_pos}) must appear before \"properties\" (pos {p_pos}). CLAUDE.md Non-Negotiable #7 is load-bearing for TB 2.0 score — see plan 02-08 T2 Step 6 Path B."
        );
    }
}
