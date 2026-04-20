# Changelog

All notable changes to Kay are documented here.

Kay follows [semver](https://semver.org/) with one non-standard rule:
**never cut a major release.** Any version bump that would otherwise warrant
a major (X.0.0) is treated as a minor bump instead. Breaking changes are
communicated here and in release notes with migration guidance, never by
incrementing the major version.

## [0.1.0] — 2026-04-20

**First signed release.** The provider layer is in; tool registry, agent loop,
frontends, and benchmark submission are still ahead.

### Highlights

- OpenRouter streaming provider with object-safe `Provider` trait, typed
  `AgentEvent` pipeline, and a tolerant two-pass JSON parser that never
  panics on malformed tool-call arguments.
- Signed release tags are now enforced — v0.0.x was the pre-stable exempt
  series; from v0.1.0 onward every tag must carry a GPG or SSH signature
  from an entry in `.github/allowed_signers`.

### Added

- **Phase 2 — Provider HAL + Tolerant JSON Parser** (PROV-01..08). Net-new
  crate `kay-provider-openrouter` with:
  - Object-safe `Provider` trait plus streaming `AgentEvent` (PROV-01, D-01).
  - End-to-end OpenRouter chat: `OpenRouterProvider` builder, `UpstreamClient`
    (reqwest + `reqwest_eventsource`), and SSE-to-`AgentEvent` translator with
    per-`tool_call.id` reassembly and an Anthropic-via-OpenRouter index
    backfill path (PROV-02).
  - Model allowlist with `:exacto` wire-name rewrite, control-character
    rejection, env-override, and `ascii-lowercase + trim` normalization
    (PROV-03, TM-04, TM-08).
  - `ApiKey` newtype with custom `Debug` that redacts to `<redacted>`; env
    precedence over config; `resolve_api_key` gates missing/invalid/expired
    (PROV-04, TM-01).
  - Tolerant two-pass tool-argument parser (`tool_parser.rs`): strict
    `serde_json` first, `forge_json_repair::json_repair` fallback, and a
    never-panic property-test suite. Malformed inputs emit
    `AgentEvent::ToolCallMalformed` as a data event; the stream continues
    (PROV-05, D-03).
  - 1 MiB cap on reassembled tool-call arguments with diagnostic
    `ToolCallMalformed` emission on breach (TM-06).
  - `backon`-powered exponential retry with full jitter (base 500ms, factor
    2, max 3 attempts, max delay 8s). `Retry-After` header honored on 429;
    503 uses the default schedule. `AgentEvent::Retry` is emitted before
    each backoff sleep. `EventSource` retries are disabled via `NeverRetry`
    so `backon` is the single source of retry truth (PROV-07, D-09, TM-09).
  - Per-session cost cap with `Mutex<f64>` spend accumulator. Turn-boundary
    enforcement only — mid-response streams are never aborted. `--max-usd 0`
    is rejected at builder time (PROV-06, D-10).
  - 11-variant `#[non_exhaustive]` `ProviderError` taxonomy with
    `classify_http_error` mapping: 401→`Auth{Invalid}`, 402→`Http`,
    429→`RateLimited`, 502/503/504→`ServerError`, transport→`Network`,
    cancellation→`Canceled` (PROV-08).
- **Phase 2.5 — kay-core sub-crate split.** Structural fix discovered
  during Phase 2 execution: ForgeCode's imported source was 23 crates, and
  forcing them into a single `kay-core` crate broke proc-macro
  self-reference, `include_str!` relative paths, trait object-safety, and
  visibility. Resolved by promoting each `forge_*` subtree to its own
  workspace sub-crate (preserving upstream layout) and reducing `kay-core`
  to a thin aggregator re-exporter. Verifier PASS 8/8.
- `.github/allowed_signers` — single source of truth for authorized release
  signers, committed to the repo. New signers onboard by PR.

### Changed

- **`milestone` field in `.planning/STATE.md`** renamed from the stale
  template value `v2.0.0` to `v0.1.0`. `v2.0.0` would have violated the
  never-major versioning policy; the STATE cursor now tracks the *current*
  release milestone rather than a conflated end-of-roadmap goal.
- Phase 1 checkbox in `.planning/ROADMAP.md` corrected to `[x]` and
  annotated with its 2026-04-19 v0.0.1 release date (was `[ ]` despite
  the status table showing Complete).
- CI `signed-tag-gate` job now configures `gpg.ssh.allowedSignersFile` to
  point at `.github/allowed_signers` before `git tag -v`. Previously the
  job would have failed on any SSH-signed tag because `allowedSignersFile`
  was never set on the CI runner.

### Security

- TM-01: API keys cannot leak via `Debug` — verified by unit tests on the
  `ApiKey` newtype.
- TM-04: Model IDs with control characters (`\r`, `\n`, `\t`) or non-ASCII
  input are rejected at the allowlist boundary.
- TM-06: Tool-call argument reassembly capped at 1 MiB; over-budget inputs
  produce a `ToolCallMalformed` event and drop the offending builder.
- TM-08: Wire-model rewrite always appends `:exacto` — the OpenRouter
  endpoint discipline that protects Terminal-Bench score variance.
- TM-09: `EventSource::NeverRetry` prevents 3×3 retry amplification when
  combined with `backon`.
- Release signing enforcement (GOV-05) is now live — CI rejects any
  unsigned `v*.*.*` tag outside the `v0.0.x` pre-stable series.

### Quality

- 81 tests green in `kay-provider-openrouter` (56 library unit tests, 25
  integration tests, 2 proptests).
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  clean.
- `cargo check --workspace` clean.
- Goal-backward verification: `02-VERIFICATION.md` reports 8/8 PROV-*
  requirements, 5/5 threat-model mitigations, 3/3 design decisions
  satisfied.
- Code review: 10 findings (0 Critical, 0 High, 2 Medium, 4 Low, 4 Info) —
  all Medium + Low remediated; 2 Info deferred with written rationale.
- Every release-scope commit carries `Signed-off-by:` (DCO).

### Contributors

- Shafqat Ullah (`shafqat@sourcevo.com`)

### Full changelog

85 commits since v0.0.1. See the GitHub release notes for the curated
highlight list, or `git log v0.0.1..v0.1.0` for the full set.

## [0.0.1] — 2026-04-19

First release. Unsigned internal/audit build. Phase 1 scaffolding only.

See the [v0.0.1 GitHub release](https://github.com/alo-exp/kay/releases/tag/v0.0.1)
for the detailed description.

[0.1.0]: https://github.com/alo-exp/kay/releases/tag/v0.1.0
[0.0.1]: https://github.com/alo-exp/kay/releases/tag/v0.0.1
