# Phase 1: Fork, Governance, Infrastructure - Context

**Gathered:** 2026-04-19
**Status:** Ready for planning
**Mode:** Autonomous (`--auto`) — user delegated all non-critical decisions

<domain>
## Phase Boundary

This phase delivers the **forkable, governed, buildable foundation** for Kay:

1. A Rust cargo workspace forked from ForgeCode with upstream attribution that satisfies Apache-2.0 obligations.
2. Governance scaffolding: LICENSE, NOTICE, CONTRIBUTING.md (with DCO + clean-room attestation), SECURITY.md.
3. CI enforcement: DCO signoff on every commit, signed-tag gate on release, cargo-deny, cargo-audit, workspace `cargo check --deny warnings` on macOS/Linux/Windows.
4. **The parity gate:** the unmodified fork must reproduce ≥80% on Terminal-Bench 2.0 with a pinned, archived reference transcript before any harness modification merges to `main` (EVAL-01).

**In scope:** GOV-01…07, WS-01…05, EVAL-01 (13 requirements).
**Out of scope:** any harness modification, new capabilities, KIRA techniques (Phase 3+), UI (Phase 9+). This phase ends the moment the parity baseline is recorded — nothing else.

</domain>

<decisions>
## Implementation Decisions

### Fork & Attribution

- **D-01 (Fork strategy):** Clone ForgeCode at its current `main` HEAD, copy source into Kay's workspace, preserve commit-level provenance via a single `NOTICE` entry + a tagged git commit `forgecode-parity-baseline` pointing at the imported SHA. We do **not** use `git subtree` or submodules — they introduce upstream drift complexity that v1 doesn't need. Clean-cut fork with a recorded parity SHA is enough.
- **D-02 (NOTICE content):** `NOTICE` at repo root lists: (a) ForgeCode's Apache-2.0 copyright holders (pulled from ForgeCode's own `NOTICE` / `LICENSE`), (b) a line stating "Portions of this codebase were derived from ForgeCode (https://github.com/antinomyhq/forgecode) at commit <SHA> on 2026-04-19, used under Apache-2.0", (c) Kay's own copyright line.
- **D-03 (Crate authors):** `authors = ["Kay Contributors <contributors@kay.dev>"]` in every workspace crate `Cargo.toml`. Individual contributions tracked via DCO signoff + git log — no per-crate author drift.
- **D-04 (README attribution):** `README.md` has an `## Acknowledgments` section naming ForgeCode as the base harness and Terminus-KIRA as the source of the harness techniques Kay plans to layer on in later phases. Top-of-file avoids ambiguity about origin.

### Workspace

- **D-05 (Workspace layout):** Mirror codex-rs — `kay-core`, `kay-cli`, `kay-tauri` (placeholder crate, built out in Phase 9), `kay-provider-openrouter` (placeholder, built in Phase 2), `kay-sandbox-macos` / `kay-sandbox-linux` / `kay-sandbox-windows` (placeholders, built in Phase 4). Phase 1 ships empty crate skeletons with only the public API outlines; the parity gate runs against `kay-core` = imported ForgeCode source.
- **D-06 (Rust edition):** 2024 (matches PROJECT.md locked decision).
- **D-07 (Workspace-level pinning):** Pin tokio 1.51 LTS, reqwest 0.13, rustls 0.23, serde_json, schemars, tracing at the workspace `[workspace.dependencies]` table. All child crates inherit. No per-crate duplication.
- **D-08 (cargo-deny):** Configure `deny.toml` to block GPL, AGPL, LGPL transitive deps (Apache-2.0 compatibility constraint), block `openssl` (use `rustls`), block known-vuln crates via `cargo-audit` integration.
- **D-09 (MSRV):** stable Rust, pinned to version in `rust-toolchain.toml`. Starting value = stable as of 2026-04-19. Bump only via explicit PR.

### DCO Enforcement

- **D-10 (DCO action):** `tim-actions/dco@master` + `tim-actions/get-pr-commits@v1.3.1` in `.github/workflows/ci.yml`. These are the Apache-licensed actions the Linux kernel and many ASF projects use — standard, mature, well-maintained. Already scaffolded in the existing `ci.yml` during silver-init.
- **D-11 (DCO bypass):** No bypass. Even maintainers sign off. "Emergency" commits are a security risk per pitfalls research.
- **D-12 (DCO trailer):** Standard — `Signed-off-by: Name <email>` on every commit. No custom trailers.

### Signed Tags

- **D-13 (Signature format):** Accept both GPG (RFC 4880) and SSH signatures (git 2.34+). SSH is easier for new contributors; GPG remains the baseline. `git tag -v` validates both.
- **D-14 (CI enforcement):** The `signed-tag-gate` job (already scaffolded in `ci.yml`) runs `git tag -v <tag>` on any `refs/tags/v*` push. Unsigned tags fail the release pipeline.
- **D-15 (Signing key publication):** Public key(s) for release signers are published at `docs/signing-keys/` in the repo + cross-referenced in `SECURITY.md`. Rotate annually or on compromise.

### Clean-room Attestation

- **D-16 (Attestation text):** `CONTRIBUTING.md` includes a short attestation clause contributors acknowledge via DCO signoff: "By signing off, I confirm I have not had exposure to leaked Claude Code source code (`@anthropic-ai/claude-code` v2.1.88 source map leak, 2026-03-31) and that this contribution contains no code derived from that leak."
- **D-17 (Enforcement posture):** Honor-system + PR review. No automated scanner — impractical without a ground-truth corpus of the leak. The CONTRIBUTING.md text + DCO signoff is the legal shield.

### Parity Gate (EVAL-01)

- **D-18 (Benchmark harness):** `harbor-framework/terminal-bench-2` (the official TB 2.0 harness) via Docker. Pin Docker image SHAs + harness commit SHA.
- **D-19 (Models used):** OpenRouter Exacto endpoint for Claude Opus 4.6 **and** GPT-5.4 (both are models ForgeCode itself scored 81.8% on — we need to confirm ForgeCode's result reproduces on at least one before declaring parity). Parity threshold: ≥ 80% on at least one of these two models.
- **D-20 (Archival):** Reference run is archived at `.planning/phases/01-fork-governance-infrastructure/parity-baseline/` with: (a) Docker SHAs, (b) OpenRouter model+date, (c) full JSONL transcript, (d) summary score, (e) `forgecode-parity-baseline` git tag pointing to the exact commit of the forked source that produced the run.
- **D-21 (CI parity check — deferred to Phase 2+):** The parity-gate CI check is **scaffolded** in Phase 1 (job stub that reads the archived score and compares) but not fully wired until Phase 2, when the first harness modification is about to land. Rationale: running Harbor in CI is expensive; we only need the check to fire when harness changes would actually regress it.
- **D-22 (Sample budget for parity run):** Budget $100 OpenRouter credits for the initial parity baseline run (Exacto-Claude Opus 4.6 or GPT-5.4 on 89 tasks × some retry margin). Phase 12 v1 submission will use its own separate budget.

### Branch Protection

- **D-23 (main protection):** Required on GitHub: (a) DCO check green, (b) CI lint/test matrix green, (c) at least 1 approving review (solo-maintained means self-approvals from a separate account or the Anomaly Innovations org account are acceptable as interim), (d) linear history (rebase or squash merge), (e) no force-push.
- **D-24 (Tags on main only):** Release tags cut from `main` only; no tags on dev branches.

### Claude's Discretion

- Exact crate skeleton content within `kay-core` (modules, trait stubs) — emerge organically during Phase 2–5 work.
- `rust-toolchain.toml` exact version pin — whatever stable is on Phase 1 execution day.
- `deny.toml` full advisory/ban list — start from cargo-deny's default template, tune as first PRs surface false positives.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level
- `.planning/PROJECT.md` — project definition, core value, all Key Decisions
- `.planning/REQUIREMENTS.md` — Phase 1 REQ-IDs: GOV-01…07, WS-01…05, EVAL-01
- `.planning/ROADMAP.md` §Phase 1 — goal and success criteria
- `.planning/STATE.md` — current position

### Research
- `.planning/research/STACK.md` — full Rust + Tauri stack with pinned versions
- `.planning/research/ARCHITECTURE.md` — workspace split (codex-rs mirror), agent loop shape
- `.planning/research/PITFALLS.md` — 15 pitfalls; Phase 1 most relevant: fork hazards (§Fork hazards — ForgeCode specifically), CLA-vs-DCO (§Licensing), macOS sidecar notarization (TAURI-02, lands in Phase 9), signed-tag gate (GOV-05)
- `.planning/research/SUMMARY.md` — executive synthesis; §Roadmap Implications §Phase 1 is the primary reference

### Existing scaffold (to extend, not overwrite)
- `CLAUDE.md` — Kay-specific GSD workflow enforcement + non-negotiables (already committed)
- `silver-bullet.md` — SB enforcement instructions (already committed)
- `.github/workflows/ci.yml` — DCO + signed-tag gates already scaffolded during silver-init
- `.gitignore` — Rust/Tauri defaults already in place
- `docs/CICD.md` — pipeline spec already drafted
- `docs/PRD-Overview.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md` — living summaries that reference .planning/

### External
- ForgeCode upstream: https://github.com/antinomyhq/forgecode (Apache-2.0, primary base)
- ForgeCode TB 2.0 81.8% writeup: https://forgecode.dev/blog/gpt-5-4-agent-improvements/ (schema hardening technique reference)
- codex-rs architecture: https://codex.danielvaughan.com/2026/03/28/codex-rs-rust-rewrite-architecture/ (workspace split inspiration)
- Harbor / Terminal-Bench 2.0: https://github.com/harbor-framework/terminal-bench-2 (parity-gate harness)
- OpenRouter Exacto endpoints: https://openrouter.ai/announcements/provider-variance-introducing-exacto (model allowlist rationale)
- DCO text: https://developercertificate.org/ (standard v1.1, no modifications)
- tim-actions/dco: https://github.com/tim-actions/dco (DCO GitHub Action)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (already in Kay)

- `.github/workflows/ci.yml` — scaffolded with DCO, lint, test matrix, signed-tag gate, frontend stub. **Extend**, don't rewrite. Add the parity-gate job as a new job in this workflow.
- `.gitignore` — Rust/Tauri/frontend defaults present.
- `docs/CICD.md` — pipeline spec referenced by `ci.yml`.
- `.planning/` — full GSD state; no changes needed by Phase 1.

### Established Patterns (from silver-init)

- File naming: UPPERCASE.md for top-level docs (CLAUDE.md, CHANGELOG.md, ARCHITECTURE.md, TESTING.md, CICD.md), lowercase/kebab-case for scaffolding.
- Commit style: conventional commits (`feat:`, `docs:`, `chore:`, etc.) + HEREDOC body + `Signed-off-by` + `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
- Cargo layout: workspace root has `[workspace]` table, per-crate dirs hold their own `Cargo.toml`. This follows codex-rs and is the Rust community norm.

### Integration Points

- ForgeCode source drops into `kay-core/src/` as a single import commit (preserves blame via a single squash-copy, not via vendored submodule).
- DCO action wires into the existing `.github/workflows/ci.yml` `dco` job — already scaffolded.
- Parity-gate CI job is a new top-level job in the same `ci.yml`; only runs on `workflow_dispatch` (manual) in Phase 1, automated in Phase 2+.

</code_context>

<specifics>
## Specific Ideas

- The `forgecode-parity-baseline` git tag is the single cryptographic anchor for "what is the unmodified fork". All parity comparisons reference this tag. Delete it only via documented annual tag rotation (if ever).
- Clean-room attestation wording cites the exact leak date + version to anchor the attestation in verifiable fact. Do not soften to "any leaked source" — ambiguity weakens the legal shield.
- `NOTICE` gets a machine-readable JSON twin at `NOTICE.json` (optional, Phase 1.x) so supply-chain tools can parse attribution programmatically. Not mandatory for v1.

</specifics>

<operational_dependencies>
## Operational Dependencies (require user action)

These are **not** design decisions — they're procurement/access items. Surfaced here so they don't block execution when reached. Only **D-OP-01** is execution-blocking for Phase 1; the rest are lead-time items the user should start now.

- **D-OP-01 (Phase 1 blocking):** OpenRouter API key + ~$100 budget for parity-baseline runs (GOV + EVAL-01). Required before EVAL-01 can execute. Claude can scaffold the harness; the run itself requires spend authorization.
- **D-OP-02 (Phase 1 non-blocking, Phase 11 blocking):** Apple Developer ID ($99/yr) for macOS notarization. 2-4 week onboarding. Start procurement immediately; not required to land Phase 1 artifacts.
- **D-OP-03 (Phase 1 non-blocking, Phase 11 blocking):** Azure Code Signing (or SSL.com/DigiCert KeyLocker) for Windows Authenticode. Weeks of EV certificate enrollment. Start now.
- **D-OP-04 (Phase 1 blocking for release only, not for the parity commit):** GPG or SSH key for release-tag signing. The `forgecode-parity-baseline` tag itself must be signed. User's personal key acceptable for v0.x; dedicated release key recommended pre-v1.0.
- **D-OP-05 (Phase 1 non-blocking):** GitHub org-level branch protection requires "pro"-tier GitHub (free tier allows only public-repo protection, which is sufficient for Kay since the repo is public). No action needed — just confirmation the repo stays public.

When execution reaches a blocked task, Claude will mark it `BLOCKED: D-OP-NN` in the task's SUMMARY.md and surface the blocker in the phase verification report.

</operational_dependencies>

<deferred>
## Deferred Ideas

- **JSON NOTICE twin** (`NOTICE.json`) — supply-chain tooling convenience. Defer to a later phase; not v1-critical.
- **Automated clean-room leak scanner** — would need a ground-truth corpus of the Claude Code leak. Legally and operationally fraught. Deferred indefinitely unless a vetted third-party scanner emerges.
- **Dedicated release-signing key (not user's personal)** — post-v0.1 concern. v0.x tags can be signed with the maintainer's personal key.
- **Reproducible builds** (bit-identical binaries across machines) — goal for v1+; v0.x builds are signed + attested but not strictly reproducible.

### Reviewed Todos (not folded)

None — no pending todos matched Phase 1 scope at intake.

</deferred>

---

*Phase: 01-fork-governance-infrastructure*
*Context gathered: 2026-04-19*
