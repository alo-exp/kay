# Kay — Claude Code Instructions

> **Always adhere strictly to this file and silver-bullet.md — they override all defaults.**

---

## Project Overview

**Kay** is an open-source terminal coding agent — a Rust fork of ForgeCode, hardened with Terminus-KIRA's harness techniques, and delivered through a native Tauri desktop UI.

- **Stack**: Rust (harness) + Tauri 2.x + TypeScript/React (frontend)
- **License**: Apache-2.0 + DCO (no CLA)
- **Git repo**: https://github.com/alo-exp/kay
- **Core value**: Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.

See `.planning/PROJECT.md` for the authoritative project definition.

---

## Active Workflow — GSD

This project uses the **GSD (Get Shit Done)** planning + execution workflow. Follow it strictly; it's the framework that keeps quality gates enforceable.

- **Plan before coding**: every non-trivial change goes through `/gsd-plan-phase <N>`. Planning produces a `PLAN.md` with goal, requirements mapping, task breakdown, and verification steps.
- **Discuss before planning**: ambiguous or gray-area phases start with `/gsd-discuss-phase <N>` to surface assumptions.
- **Execute through phases**: `/gsd-execute-phase <N>` wraps execution in atomic commits, deviation handling, and checkpoint protocols.
- **Verify every phase**: `/gsd-verify-work <N>` or the verifier agent confirms deliverables match success criteria *before* the phase closes.
- **Review loop**: `/gsd-code-review` after execution; fixes via `/gsd-code-review-fix`.
- **Ship only after verification**: `/gsd-ship <N>` prepares PR; never bypass the verify gate.

**Artifacts to honor:**

- `.planning/PROJECT.md` — project definition, core value, constraints
- `.planning/REQUIREMENTS.md` — v1 requirements (83 REQ-IDs); every PR must map to one or more
- `.planning/ROADMAP.md` — 12-phase plan; the source of truth for what ships when
- `.planning/STATE.md` — current phase cursor
- `.planning/config.json` — YOLO + fine granularity + parallel + quality models + all workflow agents on

---

## Non-Negotiables

These are locked by PROJECT.md Key Decisions and must not be silently reversed:

1. **Forked ForgeCode parity gate (Phase 1, EVAL-01)**: the unmodified fork must reproduce ≥ 80% on TB 2.0 *before* any harness modification merges. Baseline is captured on the `forgecode-parity-baseline` tag.
2. **No unsigned release tags**: every release tag is GPG- or SSH-signed. CI blocks unsigned tag pushes. (Motivated by the `alo-exp/silver-bullet#28` issue filed 2026-04-19.)
3. **DCO (not CLA)**: `Signed-off-by: Name <email>` on every commit; GitHub Action blocks unsigned PRs. Maintainers do not accept contributions without DCO.
4. **Clean-room contributor attestation**: contributors confirm no exposure to leaked Claude Code source (`@anthropic-ai/claude-code` v2.1.88 leak, 2026-03-31) before first merge. Kay's Rust code must not have any TypeScript-derived structure from that leak.
5. **Single merged Rust binary**: the ForgeCode harness is merged into the main Tauri binary — **no `externalBin` sidecar** (Tauri #11992 blocks macOS notarization on sidecars).
6. **Strict OpenRouter model allowlist**: target Exacto endpoints. "Supports 300+ models" is off-strategy; tool-calling variance destroys benchmark scores.
7. **ForgeCode's JSON schema hardening**: `required`-before-`properties`, flattening, explicit truncation reminders. Apply consistently to every tool schema. This is load-bearing for TB 2.0 score.

---

## Session Startup Expectations

Before starting any coding task:

1. Read `.planning/STATE.md` to find the current phase cursor.
2. Read `.planning/ROADMAP.md` to confirm the phase goal and success criteria.
3. If there's no active `PLAN.md` for the current phase, run `/gsd-plan-phase <N>` (or `/gsd-discuss-phase <N>` first if the phase has gray areas).
4. Never modify source under `src/` / crate directories without a `PLAN.md` that authorizes the change.
5. Never commit directly to `main`; all work flows through branches and PRs with signed-off commits.

---

## File Safety

- Never `rm -rf`, `git reset --hard`, or `git clean` without explicit user authorization for the specific target.
- Never overwrite existing uncommitted work; `git stash` or ask before destructive operations.
- `.planning/` docs are the project's memory — preserve them; don't rewrite history.

---

## Project-Specific Rules

<!-- Add project-specific Claude instructions here. -->
<!-- Silver Bullet enforcement lives in silver-bullet.md (do not duplicate here). -->

(None yet — project just initialized. Add guidance as patterns emerge.)

---

*Last updated: 2026-04-19 after GSD project initialization*
