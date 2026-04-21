//! Persona — YAML loader + schema + post-parse validators for the
//! `forge`, `sage`, `muse` agent profiles.
//!
//! # LOOP-03
//!
//! A persona is a *declarative* record of "who this agent is": its
//! system prompt, which tools it may call, and which model drives it.
//! Personas are YAML, not code — the Wave 4 loop selects a persona by
//! name at turn-start and uses its fields to configure the model call
//! + the tool-dispatch filter.
//!
//! ## The four required fields (no defaults, no optional)
//!
//! ```yaml
//! name: forge
//! system_prompt: "You are Forge, a code-writing agent..."
//! tool_filter:
//!   - fs_read
//!   - fs_write
//!   - execute_commands
//! model: anthropic/claude-sonnet-4.6
//! ```
//!
//! - `name` — human-readable handle (`forge`, `sage`, `muse`); matches
//!   the lookup key in `Persona::load`.
//! - `system_prompt` — the system message prepended to every model
//!   call for this persona.
//! - `tool_filter` — an allowlist of tool names; the loop refuses
//!   dispatch for any tool outside this list. Entries are validated
//!   against a live `ToolRegistry` via
//!   [`Persona::validate_against_registry`] so a YAML typo or
//!   malicious persona file cannot fabricate a tool that does not
//!   exist in the registry.
//! - `model` — OpenRouter model identifier. Must appear in Kay's
//!   launch allowlist (see
//!   `crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json`)
//!   — validated via [`Persona::validate_model`].
//!
//! ## Strictness guarantees
//!
//! - `#[serde(deny_unknown_fields)]` — any extra YAML key fails
//!   deserialization. This closes the *YAML persona injection* risk
//!   row in `05-BRAINSTORM.md` §Engineering-Lens Risks: a poisoned
//!   persona YAML cannot smuggle payload through unnamed fields that
//!   a future refactor starts reading from.
//! - No `#[serde(default)]` anywhere. All four fields are required;
//!   missing-field errors are intentional.
//! - `tool_filter` is `Vec<String>` at serde time (so malformed
//!   entries produce specific `ToolRegistry` misses at validation
//!   time rather than obscure deserialization errors).
//!
//! ## Post-parse validators
//!
//! `from_yaml_str` is only the first gate. Two more validators run
//! before a persona is accepted by the Wave 4 loop:
//!
//! - [`Persona::validate_against_registry`] — every `tool_filter`
//!   entry must resolve via `ToolRegistry::get(&ToolName::new(entry))`.
//! - [`Persona::validate_model`] — `model` must appear in the
//!   caller-provided allowlist slice. The allowlist is passed in
//!   rather than embedded so `kay-core` does not reverse-depend on
//!   `kay-provider-openrouter`.
//!
//! Future waves (T3.5 bundled loader, T3.6 external YAML path loader,
//! T3.7 insta snapshots) extend this module. T3.1 (tests) and T3.2
//! (this file) lock only the schema contract + post-parse validators.

use std::path::Path;

use forge_domain::ToolName;
use kay_tools::ToolRegistry;
use serde::Deserialize;

/// A parsed persona definition.
///
/// Construct via [`Persona::from_yaml_str`]; validate via
/// [`Persona::validate_against_registry`] and
/// [`Persona::validate_model`] before handing to the agent loop.
///
/// The `PartialEq + Eq` derives are purely for test ergonomics
/// (equality on the deserialized form is used by T3.7 snapshot
/// scaffolding); `Clone` keeps the type cheap to fan out per-turn.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Persona {
    /// Human-readable persona handle (e.g. "forge").
    pub name: String,

    /// System prompt prepended to every model call for this persona.
    pub system_prompt: String,

    /// Allowlist of tool names this persona may call. Each entry is
    /// validated against a live [`ToolRegistry`] by
    /// [`Persona::validate_against_registry`].
    pub tool_filter: Vec<String>,

    /// OpenRouter model identifier. Validated against the launch
    /// allowlist by [`Persona::validate_model`].
    pub model: String,
}

impl Persona {
    /// Parse a YAML document into a `Persona`.
    ///
    /// Applies `deny_unknown_fields` strictness: any extra key fails.
    /// Missing required fields fail with "missing field" errors.
    ///
    /// # Errors
    ///
    /// Returns [`PersonaError::Yaml`] on any serde/YAML failure. The
    /// underlying message names the offending field when applicable.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, PersonaError> {
        serde_yml::from_str::<Self>(yaml).map_err(PersonaError::Yaml)
    }

    /// Verify every `tool_filter` entry resolves to a tool registered
    /// in `registry`.
    ///
    /// Fails fast on the first unknown name — ordering matches the
    /// YAML order as read by serde.
    ///
    /// # Errors
    ///
    /// Returns [`PersonaError::UnknownTool`] naming the first entry
    /// missing from the registry.
    pub fn validate_against_registry(&self, registry: &ToolRegistry) -> Result<(), PersonaError> {
        for entry in &self.tool_filter {
            let name = ToolName::new(entry);
            if registry.get(&name).is_none() {
                return Err(PersonaError::UnknownTool(entry.clone()));
            }
        }
        Ok(())
    }

    /// Verify `self.model` appears in the caller-provided allowlist.
    ///
    /// Comparison is exact-string (no case folding, trimming, or
    /// normalization). The caller is responsible for passing an
    /// allowlist that matches the provider's wire-form.
    ///
    /// # Errors
    ///
    /// Returns [`PersonaError::ModelNotAllowed`] carrying the rejected
    /// model id if it is not in `allowlist`.
    pub fn validate_model(&self, allowlist: &[&str]) -> Result<(), PersonaError> {
        if allowlist.iter().any(|m| *m == self.model.as_str()) {
            Ok(())
        } else {
            Err(PersonaError::ModelNotAllowed(self.model.clone()))
        }
    }

    /// Load a bundled persona by name.
    ///
    /// Resolves `name` against the three YAMLs bundled at compile
    /// time from `crates/kay-core/personas/`:
    ///
    /// | `name` value | Bundled source                |
    /// |--------------|-------------------------------|
    /// | `"forge"`    | `personas/forge.yaml`         |
    /// | `"sage"`     | `personas/sage.yaml`          |
    /// | `"muse"`     | `personas/muse.yaml`          |
    ///
    /// The bundling uses `include_str!`, so the YAML content is
    /// baked into the `kay-core` binary — no filesystem lookup at
    /// runtime, no working-directory sensitivity, no missing-file
    /// errors at load time. This matches the CLI-01 "works out of
    /// the box" requirement.
    ///
    /// # Errors
    ///
    /// - [`PersonaError::UnknownPersona`] — `name` does not match a
    ///   bundled persona. The kay-cli layer catches this to offer
    ///   the user a suggestion ("try forge / sage / muse").
    /// - [`PersonaError::Yaml`] — the bundled YAML failed to parse.
    ///   This should not happen in production because the YAMLs are
    ///   authored in-tree and validated by the T3.7 snapshot tests,
    ///   but the error is surfaced rather than panicked for defense
    ///   in depth.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kay_core::persona::Persona;
    ///
    /// let forge = Persona::load("forge").expect("forge is bundled");
    /// assert_eq!(forge.name, "forge");
    /// ```
    pub fn load(name: &str) -> Result<Self, PersonaError> {
        let yaml = match name {
            "forge" => include_str!("../personas/forge.yaml"),
            "sage" => include_str!("../personas/sage.yaml"),
            "muse" => include_str!("../personas/muse.yaml"),
            _ => return Err(PersonaError::UnknownPersona(name.to_string())),
        };
        Self::from_yaml_str(yaml)
    }

    /// Load a persona from an external YAML file on disk.
    ///
    /// Extension point for Phase 11+ power users who want to drop a
    /// custom persona YAML into e.g. `~/.config/kay/personas/<name>.yaml`
    /// without recompiling `kay-core`. The bundled loader
    /// ([`Persona::load`]) handles the opinionated default triplet
    /// (forge / sage / muse); `from_path` handles everything else.
    ///
    /// Schema strictness is identical to the bundled loader — the same
    /// `#[serde(deny_unknown_fields)]` + required-field gates apply.
    /// An external YAML cannot grant itself laxer parsing than a
    /// bundled one.
    ///
    /// # Errors
    ///
    /// - [`PersonaError::Io`] — the path could not be read (missing
    ///   file, permission denied, not a regular file, etc.). Carries
    ///   the underlying `std::io::Error` for caller inspection.
    /// - [`PersonaError::Yaml`] — the file was read but did not parse
    ///   as a valid `Persona` (missing required field, unknown field,
    ///   malformed YAML, wrong types, etc.).
    ///
    /// Isolating I/O failures from parse failures lets the CLI
    /// differentiate "file not found — check the path" from "file is
    /// malformed — check the schema" when surfacing errors to users.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kay_core::persona::Persona;
    /// use std::path::Path;
    ///
    /// let custom = Persona::from_path(Path::new("/etc/kay/custom.yaml"))
    ///     .expect("custom persona must load");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, PersonaError> {
        let yaml = std::fs::read_to_string(path.as_ref()).map_err(PersonaError::Io)?;
        Self::from_yaml_str(&yaml)
    }
}

/// Error surface for persona loading and post-parse validation.
///
/// Extended in Wave 3 later tasks (T3.6 adds `Io` for the
/// external-path loader). T3.1 / T3.2 cover YAML parse + two
/// validation branches; T3.5 adds `UnknownPersona` for the bundled
/// lookup.
#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    /// YAML parse or schema error — includes deny_unknown_fields
    /// violations and missing-required-field errors.
    #[error("persona YAML parse error: {0}")]
    Yaml(serde_yml::Error),

    /// A `tool_filter` entry is not present in the supplied
    /// `ToolRegistry`. The inner string is the offending entry.
    #[error("tool '{0}' in tool_filter is not registered in the ToolRegistry")]
    UnknownTool(String),

    /// The `model` field is not in the caller-provided launch
    /// allowlist. The inner string is the rejected model id.
    #[error("model '{0}' is not in Kay's launch allowlist")]
    ModelNotAllowed(String),

    /// `Persona::load(name)` was called with a name that does not
    /// match any of the bundled personas (forge / sage / muse).
    /// The inner string is the rejected name so the CLI can echo it
    /// back to the user ("persona 'ghost' is not bundled; try forge,
    /// sage, or muse").
    #[error("persona '{0}' is not bundled (expected one of: forge, sage, muse)")]
    UnknownPersona(String),

    /// `Persona::from_path(p)` failed to read the file at `p`
    /// (missing file, permission denied, not a regular file, etc.).
    /// Held separately from `Yaml` so the CLI can distinguish
    /// "file not found — check the path" from "file is malformed —
    /// check the schema" when surfacing errors.
    #[error("persona I/O error: {0}")]
    Io(std::io::Error),
}
