//! Exit-code classifier for `kay` (Phase 5 Wave 7 T7.7).
//!
//! Closes REQ **CLI-03**: the CLI's exit codes must reflect task
//! success, failure mode, and abort reason distinctly enough that
//! shell scripts and CI runners can act on them without parsing
//! stderr. Five codes are defined:
//!
//! | Code | Variant               | Meaning                                                                                                       |
//! |------|-----------------------|---------------------------------------------------------------------------------------------------------------|
//! |   0  | `Success`             | A turn completed cleanly. Any `AgentEvent::TaskComplete { verified: true, Pass }` observed; no sandbox hits.  |
//! |   1  | `RuntimeError`        | Generic runtime failure — provider error, max-turns budget exceeded, join error, etc. The fallback mapping.   |
//! |   2  | `SandboxViolation`    | One or more `AgentEvent::SandboxViolation` frames landed on the JSONL stream. Exit AFTER draining the stream. |
//! |   3  | `ConfigError`         | Invalid configuration at startup (bad persona path, malformed YAML, unknown bundled name, unknown tool, …).   |
//! | 130  | `UserAbort`           | SIGINT routed through T7.8's handler → `ControlMsg::Abort` → exit. POSIX convention is `128 + SIGINT (=2)`.   |
//!
//! # Why not use `anyhow::Result<()>`'s default `Termination`?
//!
//! The stdlib default maps `Err(_)` → exit 1 with no distinction
//! between "max turns" and "persona YAML is malformed" and "sandbox
//! said no". Every tier above exit-1 (2, 3, 130) needs explicit
//! classification, so `main` takes over the exit-code decision
//! itself: `main()` returns `()`, and `std::process::exit(code)` is
//! the last statement before drop.
//!
//! # QG-C4 interaction
//!
//! `SandboxViolation` is a user-visible signal that MUST flow through
//! the JSONL stream on stdout (the frontend's detection surface). The
//! exit code 2 is the SHELL's detection surface. They are independent:
//! a broken-pipe on stdout (e.g., `kay run … | head -n 1`) can suppress
//! the JSONL event from reaching the shell, but the loop's internal
//! state still triggers exit 2 because the classifier runs AFTER the
//! drain loop completes. Conversely, the JSONL stream is NEVER
//! re-injected into model context — the `kay-core::event_filter`
//! (100 %-coverage gated) strips `SandboxViolation` before it reaches
//! the Responses-API compose path. Two channels, two surfaces, one
//! invariant.

use kay_core::persona::PersonaError;

/// Canonical exit codes emitted by `kay`. `#[repr(u8)]` so the
/// integer value is stable across the ABI — shell scripts, CI, and
/// the upstream `exit_code_*` E2E tests all read the raw number.
///
/// The discriminants match the CLI-03 contract table in the module
/// docstring. DO NOT reorder; DO NOT change discriminants without a
/// REQ-level spec bump — downstream consumers treat these as wire
/// codes (shell `$?`, `assert_cmd::Assert::code(N)`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    RuntimeError = 1,
    SandboxViolation = 2,
    ConfigError = 3,
    /// SIGINT observed and cleanly propagated through the control
    /// channel as `ControlMsg::Abort`. Constructed by T7.8's signal
    /// handler; intentionally unconstructed in T7.7 (no SIGINT route
    /// exists yet). The `#[allow]` below must be removed when T7.8
    /// lands so the variant's dead-code check reactivates.
    #[allow(dead_code)]
    UserAbort = 130,
}

impl ExitCode {
    /// Returns the raw `u8` discriminant.
    ///
    /// `std::process::exit` takes `i32`, but the surface here stays
    /// `u8` so the wire contract is enforced at the type level:
    /// discriminants ≥ 256 can't be represented and would fail to
    /// compile. Callers widen to `i32` at the `process::exit(…)`
    /// call site, not here.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Classifies an `anyhow::Error` produced by one of the subcommand
/// executors (`run::execute`, `eval::run`, …) into the appropriate
/// `ExitCode`.
///
/// Rules:
///
///   * If the error's chain contains a [`PersonaError`] → `ConfigError`
///     (3). Persona errors cover every known config-error tier:
///     missing file (`Io`), malformed YAML (`Yaml`), unknown bundled
///     name (`UnknownPersona`), validation failures (`UnknownTool` /
///     `ModelNotAllowed`).
///   * Otherwise → `RuntimeError` (1). This is the conservative
///     bucket: max-turns bail, provider transport failure, join
///     error, broken-pipe on stdout, etc. The E2E test suite asserts
///     exact codes on specific scenarios, and any new error class
///     that needs its own code will add a branch here plus a test.
///
/// `SandboxViolation` (exit 2) and `UserAbort` (exit 130) are NOT
/// set via this classifier. They originate from observing specific
/// runtime states (a frame on the event channel; a SIGINT handler
/// firing) and are returned as `Ok(ExitCode::SandboxViolation)` /
/// `Ok(ExitCode::UserAbort)` directly by the subcommand executor.
/// The classifier only runs on the `Err(_)` arm.
pub fn classify_error(e: &anyhow::Error) -> ExitCode {
    // Walk the chain — a `?` in `run.rs` may have wrapped
    // `PersonaError` inside one or more `anyhow::Context` layers, so
    // `downcast_ref` at the top level would miss nested cases.
    // `chain()` yields every source in order (outermost first).
    for cause in e.chain() {
        if cause.downcast_ref::<PersonaError>().is_some() {
            return ExitCode::ConfigError;
        }
    }
    ExitCode::RuntimeError
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn exit_code_discriminants_match_cli_contract() {
        // Locks the CLI-03 wire contract at the discriminant level.
        // Changes to these numbers require a REQ-level spec bump +
        // the E2E `exit_code_*` assertions below flipping in lockstep.
        assert_eq!(ExitCode::Success.as_u8(), 0);
        assert_eq!(ExitCode::RuntimeError.as_u8(), 1);
        assert_eq!(ExitCode::SandboxViolation.as_u8(), 2);
        assert_eq!(ExitCode::ConfigError.as_u8(), 3);
        assert_eq!(ExitCode::UserAbort.as_u8(), 130);
    }

    #[test]
    fn classify_persona_io_error_is_config_error() {
        // The `exit_code_3_on_config_error` E2E spawns kay with a
        // nonexistent `--persona PATH`. Persona loader returns
        // `PersonaError::Io(…)` which `?`-converts to
        // `anyhow::Error`. Classifier must see through the wrap.
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
        let e: anyhow::Error = PersonaError::Io(io).into();
        assert_eq!(classify_error(&e), ExitCode::ConfigError);
    }

    #[test]
    fn classify_persona_unknown_is_config_error() {
        // `--persona` not required — `Persona::load("ghost")` also
        // yields a ConfigError because the name isn't bundled.
        let e: anyhow::Error = PersonaError::UnknownPersona("ghost".into()).into();
        assert_eq!(classify_error(&e), ExitCode::ConfigError);
    }

    #[test]
    fn classify_persona_yaml_is_config_error() {
        // Malformed YAML → Yaml variant. Same bucket (config error)
        // because the user can fix the file and retry — the agent
        // itself didn't fail.
        let yaml_err = serde_yml::from_str::<serde_yml::Value>("{ invalid: [unclosed").unwrap_err();
        let e: anyhow::Error = PersonaError::Yaml(yaml_err).into();
        assert_eq!(classify_error(&e), ExitCode::ConfigError);
    }

    #[test]
    fn classify_persona_through_context_is_config_error() {
        // `anyhow::Error::context` wraps the original error in a new
        // outer layer. The classifier walks the chain so the nested
        // PersonaError still resolves to ConfigError even when it
        // isn't the top-level cause. `.context(...)` is an inherent
        // method on `anyhow::Error` — no trait import needed.
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
        let inner: anyhow::Error = PersonaError::Io(io).into();
        let outer = inner.context("loading persona from /etc/secret.yaml");
        assert_eq!(classify_error(&outer), ExitCode::ConfigError);
    }

    #[test]
    fn classify_generic_error_is_runtime_error() {
        // Everything that isn't a PersonaError falls through to
        // exit 1. A bare `anyhow::anyhow!` is the simplest case.
        let e = anyhow::anyhow!("max turns exceeded (budget: 0)");
        assert_eq!(classify_error(&e), ExitCode::RuntimeError);
    }

    #[test]
    fn classify_stdio_error_is_runtime_error() {
        // Broken-pipe on stdout (e.g. `kay run … | head -n 1`)
        // bubbles up as `std::io::Error` → anyhow. Not a config
        // problem; classifier must route it to RuntimeError (1),
        // not ConfigError (3). Covers the future broken-pipe path
        // T7.7 docstring mentions.
        let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "downstream closed");
        let e: anyhow::Error = io.into();
        assert_eq!(classify_error(&e), ExitCode::RuntimeError);
    }
}
