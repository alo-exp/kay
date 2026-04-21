# `AgentEvent` JSONL Wire Schema — v1

**Status:** Frozen at Phase 5 (Kay v0.3.0, 2026-04-21).
**Schema version:** `1`.
**Source:** `crates/kay-tools/src/events.rs` (runtime type) →
`crates/kay-tools/src/events_wire.rs` (wire serialization).
**Enforced by:** 21 `insta` snapshots in
`crates/kay-tools/tests/events_wire_snapshots.rs` (CI-blocking on all 3 OSes).

This document is the human-readable spec of the bytes that flow over
the newline-delimited JSON (JSONL) stream from `kay-cli run --prompt`
to its consumers: the Tauri GUI (Phase 9), the `ratatui` TUI
(Phase 9.5), and any third-party integration. It is the **contract**
between the Kay harness and every frontend that will ever consume
it — downstream tools are allowed to encode it into their own type
systems and assume it will not silently change.

---

## 1. Stream shape

One JSON object per event, terminated by exactly one `\n`:

```
{"type":"text_delta","content":"Hello"}\n
{"type":"tool_call_start","id":"call_01","name":"execute_commands"}\n
{"type":"usage","prompt_tokens":1024,"completion_tokens":256,"cost_usd":0.0025}\n
```

Rules:

1. **One event per line.** Any `\n` embedded in a string field is
   JSON-escaped as `\\n` — the outer delimiter is always a literal
   newline. Consumers may use `BufRead::read_line` (or equivalent)
   without a streaming JSON parser.
2. **Single trailing newline.** Not zero, not two. The Kay harness
   emits `writeln!` exactly once per event.
3. **UTF-8, no BOM.** All string fields are valid UTF-8 per RFC 8259.
4. **Compact encoding.** No pretty-printing; no trailing whitespace
   inside the JSON object.
5. **Field-order is not guaranteed.** Consumers MUST NOT rely on
   key order within the object. (The insta snapshots sort keys for
   deterministic diffs; the runtime encoder does not.)

The terminal event is always one of: `Usage` (natural end), `Error`
(unrecoverable), or `Aborted` (control-channel termination).

---

## 2. Variants (13 total, `#[non_exhaustive]`)

Every variant has a `type` tag in `snake_case`. Consumers SHOULD
switch on `type` and SHOULD accept unknown types by forwarding or
logging — new variants are additive schema changes (see §4).

### 2.1 `text_delta`

Streaming content from the model's `content` channel.

```json
{"type":"text_delta","content":"<chunk>"}
```

| Field     | Type   | Notes |
|-----------|--------|-------|
| `type`    | string | Always `"text_delta"` |
| `content` | string | Raw UTF-8 chunk; may contain escaped `\n` |

### 2.2 `tool_call_start`

A tool call has begun; subsequent `tool_call_delta` events carry
arguments until `tool_call_complete` or `tool_call_malformed`.

```json
{"type":"tool_call_start","id":"call_01","name":"execute_commands"}
```

| Field  | Type   | Notes |
|--------|--------|-------|
| `type` | string | Always `"tool_call_start"` |
| `id`   | string | Provider-assigned call id; stable for correlation |
| `name` | string | Tool name (matches `kay-tools` registry) |

### 2.3 `tool_call_delta`

Incremental arguments for an in-progress tool call.

```json
{"type":"tool_call_delta","id":"call_01","arguments_delta":"{\"cmd\":\"ls\"}"}
```

| Field             | Type   | Notes |
|-------------------|--------|-------|
| `type`            | string | Always `"tool_call_delta"` |
| `id`              | string | Matches the `tool_call_start.id` |
| `arguments_delta` | string | Raw byte-chunk; may be empty per OpenRouter variance |

### 2.4 `tool_call_complete`

Tool call fully assembled with valid JSON arguments.

```json
{"type":"tool_call_complete","id":"call_01","name":"execute_commands","arguments":{"cmd":"ls -la","cwd":"/tmp"}}
```

| Field       | Type        | Notes |
|-------------|-------------|-------|
| `type`      | string      | Always `"tool_call_complete"` |
| `id`        | string      | Matches the `tool_call_start.id` |
| `name`      | string      | Tool name |
| `arguments` | JSON object | Parsed arguments (any valid JSON) |

### 2.5 `tool_call_malformed`

Tool arguments did not parse even after `forge_json_repair` fallback.
Consumers should surface this to the user; **do not execute**.

```json
{"type":"tool_call_malformed","id":"call_02","raw":"{broken-json","error":"expected '}' at position 12"}
```

| Field   | Type   | Notes |
|---------|--------|-------|
| `type`  | string | Always `"tool_call_malformed"` |
| `id`    | string | Matches the `tool_call_start.id` |
| `raw`   | string | Original bytes for diagnosis |
| `error` | string | Parser error message |

### 2.6 `usage`

Usage/cost report emitted at turn end (per OpenRouter streaming docs,
usage arrives on the final chunk).

```json
{"type":"usage","prompt_tokens":1024,"completion_tokens":256,"cost_usd":0.0025}
```

| Field               | Type   | Notes |
|---------------------|--------|-------|
| `type`              | string | Always `"usage"` |
| `prompt_tokens`     | u64    | Input tokens |
| `completion_tokens` | u64    | Output tokens |
| `cost_usd`          | f64    | USD cost (may be `0.0` for free tiers) |

### 2.7 `retry`

A retryable upstream error is being retried after `delay_ms`. Emitted
**before** the backoff sleep so UIs can render progress.

```json
{"type":"retry","attempt":2,"delay_ms":1500,"reason":"rate_limited"}
```

| Field      | Type   | Notes |
|------------|--------|-------|
| `type`     | string | Always `"retry"` |
| `attempt`  | u32    | 1-indexed retry count |
| `delay_ms` | u64    | Backoff before next attempt |
| `reason`   | string | Snake-case tag; see §3.1 |

### 2.8 `error`

Terminal, non-retryable error. The stream ends immediately after.

```json
{"type":"error","kind":"http","message":"HTTP 503: upstream overloaded"}
```

| Field     | Type   | Notes |
|-----------|--------|-------|
| `type`    | string | Always `"error"` |
| `kind`    | string | Snake-case tag; see §3.2 |
| `message` | string | Human-readable message. **Source-error internals (reqwest line numbers, serde_json positions) are stripped** to keep snapshots deterministic across crate-version bumps. |

### 2.9 `tool_output`

Streamed output chunk from a running tool. One event per chunk plus a
terminal `closed` chunk.

```json
{"type":"tool_output","call_id":"call_01","chunk":{"kind":"stdout","data":"line of output\n"}}
{"type":"tool_output","call_id":"call_01","chunk":{"kind":"stderr","data":"warning: unused variable\n"}}
{"type":"tool_output","call_id":"call_01","chunk":{"kind":"closed","exit_code":0,"marker_detected":true}}
```

| Field     | Type   | Notes |
|-----------|--------|-------|
| `type`    | string | Always `"tool_output"` |
| `call_id` | string | Matches the originating `tool_call_complete.id` |
| `chunk`   | object | Discriminated by `kind`; see below |

`chunk.kind` values:

- `"stdout"` — `{kind, data: string}`
- `"stderr"` — `{kind, data: string}`
- `"closed"` — `{kind, exit_code: i32 | null, marker_detected: bool}`

### 2.10 `task_complete`

Terminal signal from the `task_complete` tool, carrying a
verification outcome.

```json
{"type":"task_complete","call_id":"call_03","verified":false,"outcome":{"status":"pending","reason":"Multi-perspective verification wired in Phase 8 (VERIFY-01..04)"}}
```

| Field      | Type    | Notes |
|------------|---------|-------|
| `type`     | string  | Always `"task_complete"` |
| `call_id`  | string  | Matches the originating tool call |
| `verified` | boolean | Verifier result (Phase 3 uses `NoOpVerifier` → always `false`) |
| `outcome`  | object  | Discriminated by `status`; see below |

`outcome.status` values:

- `"pending"` — `{status, reason: string}` (Phase 3 / Phase 5 default)
- `"pass"`    — `{status, note: string}` (Phase 8 real verifier)
- `"fail"`    — `{status, reason: string}` (Phase 8 real verifier)

### 2.11 `image_read`

Raw image bytes were read by an image tool. **The raw bytes are NOT
inlined** — they can be multi-MiB and would make the JSONL stream
unusable for real-time UIs. Consumers that need the payload should
re-invoke the tool locally on the same `path`.

```json
{"type":"image_read","path":"/tmp/screenshot.png","size_bytes":8,"encoding":"base64"}
```

| Field        | Type   | Notes |
|--------------|--------|-------|
| `type`       | string | Always `"image_read"` |
| `path`       | string | Absolute path the tool resolved |
| `size_bytes` | u64    | Payload length in bytes |
| `encoding`   | string | Always `"base64"` (reserved for future formats) |

Invariant (tested by `events_wire::tests::wire_never_leaks_image_bytes`):
**the wire form MUST NOT contain a `bytes` field or inline the raw
content.**

### 2.12 `sandbox_violation`

A sandbox policy violation was detected for a tool call.
**Security-critical:** consumers MUST treat this as a diagnostic
event and MUST NOT re-feed it into the model's message history
(QG-C4 carry-forward from Phase 4 — prompt-injection surface).

```json
{"type":"sandbox_violation","call_id":"call_04","tool_name":"fs_write","resource":"/etc/passwd","policy_rule":"write-outside-project-root","os_error":13}
```

| Field         | Type          | Notes |
|---------------|---------------|-------|
| `type`        | string        | Always `"sandbox_violation"` |
| `call_id`     | string        | Matches the originating tool call |
| `tool_name`   | string        | Tool that attempted the violation |
| `resource`    | string        | Path/host/port the tool tried to access |
| `policy_rule` | string        | Stable rule id (e.g. `"write-outside-project-root"`, `"net-not-allowlisted"`) |
| `os_error`    | i32 \| null   | `null` = pre-flight userspace check; non-null = kernel denial (errno) |

### 2.13 `paused`

The agent loop has paused (control channel received `Pause`). Events
between `paused` and the next non-paused event are **replayed from
the loop's internal buffer** — the stream implicitly signals resume
by emitting them.

```json
{"type":"paused"}
```

No payload.

### 2.14 `aborted`

The agent loop terminated non-recoverably before natural turn-end.

```json
{"type":"aborted","reason":"user_ctrl_c"}
```

| Field    | Type   | Notes |
|----------|--------|-------|
| `type`   | string | Always `"aborted"` |
| `reason` | string | Snake-case tag; see §3.3 |

---

## 3. Stable tag enumerations

### 3.1 `retry.reason`

| Tag              | Origin                                   |
|------------------|------------------------------------------|
| `rate_limited`   | `ProviderError::RateLimited`             |
| `server_error`   | `ProviderError::ServerError`             |
| `unknown`        | Future `RetryReason` variants (fallback) |

### 3.2 `error.kind`

| Tag                      | Origin                                   |
|--------------------------|------------------------------------------|
| `network`                | `ProviderError::Network`                 |
| `http`                   | `ProviderError::Http`                    |
| `rate_limited`           | `ProviderError::RateLimited`             |
| `server_error`           | `ProviderError::ServerError`             |
| `auth`                   | `ProviderError::Auth`                    |
| `model_not_allowlisted`  | `ProviderError::ModelNotAllowlisted`     |
| `cost_cap_exceeded`      | `ProviderError::CostCapExceeded`         |
| `tool_call_malformed`    | `ProviderError::ToolCallMalformed`       |
| `serialization`          | `ProviderError::Serialization`           |
| `stream`                 | `ProviderError::Stream`                  |
| `canceled`               | `ProviderError::Canceled`                |
| `unknown`                | Future variants (fallback)               |

### 3.3 `aborted.reason`

| Tag                             | Emitted when                                                      |
|---------------------------------|-------------------------------------------------------------------|
| `user_ctrl_c`                   | Ctrl-C cooperative abort after 2s grace                           |
| `max_turns_exceeded`            | Turn-budget safety net fires (LOOP-04)                            |
| `verifier_fail`                 | `task_complete` returned `verified=false` and policy is fail-fast |
| `sandbox_violation_propagated`  | Sandbox denial count crossed abort threshold (QG-C4 terminator)   |

Adding a new reason is an additive schema change (see §4).

---

## 4. Schema drift policy

Any change to this wire schema requires **all three** of:

1. **Snapshot update.** Run `cargo insta review` and accept the new
   `.snap` file(s) in `crates/kay-tools/tests/snapshots/`. CI blocks
   merges with stale or un-reviewed snapshots on all 3 OSes.
2. **This document bumped.** Update the relevant section and the
   **Schema version** at the top. If the change is breaking (field
   removed, rename, type change, tag removed), bump to v2 and
   enumerate the migration path.
3. **`kay-cli --version` reports the new schema version.** The CLI
   prints `event-schema: N` alongside its Kay version so consumers
   can refuse to connect to incompatible streams.

### What is "additive" (safe, stays v1)

- New variant (`type` value not yet listed here)
- New optional field on an existing variant (consumers MUST ignore
  unknown fields — Rust's `serde_json` does by default)
- New value in a `reason` / `kind` / `status` enum (consumers MUST
  handle unknown tags gracefully — fall through to a generic
  display)

### What is "breaking" (requires v2 bump)

- Renaming a `type` tag
- Removing a variant
- Removing or renaming a field
- Changing a field's JSON type (string → number, etc.)
- Removing an enum value (even if replaced with an alias)

### What to do if you need a breaking change

1. Raise it on the issue tracker with "schema-break" label.
2. Propose a v2 spec alongside v1.
3. `kay-cli` will support both versions for ≥ 1 milestone release
   (a deprecation window), emitting a warning banner when consumers
   negotiate v1.
4. Remove v1 support only after the deprecation window closes.

---

## 5. Machine-enforced lock

Every variant and every reason-tag in this document has a
corresponding `insta` snapshot in
`crates/kay-tools/tests/snapshots/events_wire_snapshots__<name>.snap`.
CI runs `cargo test -p kay-tools --test events_wire_snapshots` on
macOS, Linux, and Windows; a failed snapshot is a CI block.

Current snapshot count: **21** (16 variants + 5 Aborted reason tags).

Full list:

- `snap_text_delta`
- `snap_tool_call_start`
- `snap_tool_call_delta`
- `snap_tool_call_complete`
- `snap_tool_call_malformed`
- `snap_usage`
- `snap_retry`
- `snap_error`
- `snap_tool_output_stdout`, `snap_tool_output_stderr`, `snap_tool_output_closed`
- `snap_task_complete`
- `snap_image_read`
- `snap_sandbox_violation`, `snap_sandbox_violation_preflight`
- `snap_paused`
- `snap_aborted_user_ctrl_c`, `snap_aborted_max_turns`, `snap_aborted_verifier_fail`, `snap_aborted_sandbox_violation_propagated`
- `snap_jsonl_line_format` (Display-impl line-framing lock)

---

## 6. Reference implementations

- **Producer (Kay):** `crates/kay-tools/src/events_wire.rs` — the
  `AgentEventWire<'a>` newtype with `serde::Serialize` and
  `std::fmt::Display` (JSONL line emitter).
- **Consumer (kay-cli):** `crates/kay-cli/src/stream.rs` (Wave 7).
- **Consumer (Tauri GUI):** Phase 9, TBD.
- **Consumer (TUI):** Phase 9.5, TBD.

---

## 7. Change log

| Version | Date       | Change                                              |
|---------|------------|-----------------------------------------------------|
| v1      | 2026-04-21 | Initial freeze (Phase 5 Wave 1): 13 variants, 3 reason-tag enumerations, 21 snapshot locks. |
