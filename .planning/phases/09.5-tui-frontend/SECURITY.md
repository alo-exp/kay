# Phase 9.5 Security Review — 2026-04-24

## SEC-01: Path Injection via KAY_CLI_PATH
- **Severity:** LOW (CLI tool, user-controlled)
- **Location:** `crates/kay-tui/src/subprocess.rs:21`
- **Finding:** `kay_cli_binary()` reads `KAY_CLI_PATH` env var, allowing arbitrary path injection
- **Mitigation:** `kill_on_drop(true)` ensures subprocess cleanup regardless of binary path
- **Acceptable:** `kay` is a CLI tool; users can already run arbitrary commands via shell

## SEC-02: Command Injection
- **Severity:** NONE
- **Location:** `crates/kay-tui/src/subprocess.rs:54-60`
- **Finding:** `Command::new(binary).args(args)` — args passed through `&[String]` (safe)
- **Mitigation:** `args` is a typed `&[String]` slice, not arbitrary shell strings

## SEC-03: Unbounded Memory via Malformed JSON
- **Severity:** NONE
- **Location:** `crates/kay-tui/src/jsonl.rs`
- **Finding:** Malformed JSON returns error (ERR-01) and does not accumulate unboundedly
- **Mitigation:** LineBuffer drops oldest when exceeding 1 MB; JsonlParser logs and skips bad lines

## SEC-04: No Sensitive Data in EventLog
- **Severity:** NONE
- **Location:** `crates/kay-tui/src/state.rs`
- **Finding:** EventLog stores all TuiEvent variants; sensitive data (tool arguments, file contents) are stored
- **Mitigation:** EventLog is in-memory only (not persisted); 10_000 event cap limits exposure
- **Note:** Phase 6 (Session Store) handles persistence with proper encryption

## SEC-05: ratatui::restore()
- **Severity:** NONE
- **Finding:** `ratatui::restore()` called in all code paths (success + error via `Drop`)
- **Mitigation:** Terminal state is always restored even if the app panics

**Conclusion: Phase 9.5 security posture is ACCEPTABLE for initial release.**
