# Phase 5 Context — Locked Decisions for Planner

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop + Canonical CLI
> **Mode:** autonomous (§10e) — inline `gsd-discuss-phase` execution; all open questions resolved deterministically from upstream artifacts + codebase facts
> **Skill:** `gsd-discuss-phase` (workflow.discuss_mode=discuss)
> **Inputs scanned:** 05-BRAINSTORM.md, 05-TEST-STRATEGY.md, 05-IMPL-OUTLINE.md, 05-VALIDATION.md, 05-QUALITY-GATES.md, crates/forge_main/, git tags, .planning/config.json

---

## Purpose

Lock decisions so downstream agents (`gsd-analyze-dependencies`, `gsd-plan-phase`) can act without asking the user again. Addresses the 5 open questions from BRAINSTORM §Product-Lens + 3 INFO items from VALIDATION + any plan-time-critical decisions discovered during discuss scouting.

---

## Prior context reused (skip re-asking)

| Source | Locked facts — DO NOT re-ask |
| ------ | ---------------------------- |
| PROJECT.md Key Decisions | Forked ForgeCode parity gate ≥ 80% TB 2.0; signed tags; DCO; clean-room; single merged binary; strict OpenRouter allowlist; JSON schema hardening |
| STATE.md | Milestone v0.3.0 in_progress; Phase 4 complete (1ae2a7f squash merge); Phase 5 active |
| ROADMAP.md Phase 5 | 11 REQs in scope (LOOP-01..06 + CLI-01/03/04/05/07); 8 success criteria; depends on Phases 2/3/4 |
| config.json | discuss_mode=discuss; model_profile=quality; parallelization=true; tdd_mode=false (but TDD enforced via superpowers:test-driven-development skill per /silver:feature pipeline); branching_strategy=none (but phase branch already exists: phase/05-agent-loop) |
| BRAINSTORM §Product-Lens | 4 personas defined; 8 metrics set; 9 risks mitigated; scope IN/OUT locked |
| BRAINSTORM §Engineering-Lens | E1-E12 decisions locked (4-channel tokio::select!; AgentEventWire; YAML personas; sage_query; kay-cli clap shape; forge_main port strategy; R-1; R-2; trybuild; event_filter for QG-C4) |
| TEST-STRATEGY.md | 11 test suites T-1..T-11; coverage matrix; 3-OS CI matrix; event_filter = 100% line + 100% branch threshold (SHIP BLOCK if missed) |
| IMPL-OUTLINE.md | 7-wave breakdown; dependency DAG; commit cadence (RED+GREEN+optional REFACTOR per task; DCO on each) |
| QUALITY-GATES.md | 9 dimensions PASS design-time; 7 carry-forward enforcement contracts |

---

## Codebase scouting (this step)

**Reusable assets confirmed present:**
- `kay-core` crate (aggregator re-exporter today); Phase 5 adds new modules inline
- `kay-tools` crate with AgentEvent/ToolError/#[non_exhaustive] machinery from Phase 3
- `kay-tools::runtime::dispatcher::dispatch()` — ready-made seam for loop integration
- `kay-tools::seams::rng::{OsRngSeam, DeterministicRng}` — for deterministic property tests
- Phase 4 sandbox crates (`kay-sandbox-{policy,macos,linux,windows}`) — loop consumes via existing `Sandbox` trait; no modifications needed
- `git tag forgecode-parity-baseline` at `9985d77` — EXISTS → real parity fixture achievable
- `crates/forge_main/src/*.rs` — 35 files inventoried; port scope narrowed to 6 brand-surface files

**Files that must NOT be modified in Phase 5 (scope discipline):**
- `crates/kay-sandbox-*/src/**` — Phase 4 territory, frozen
- `crates/forge_main/src/{editor,input,state,completer,conversation_selector,...}.rs` — internal modules → Phase 10
- `crates/kay-tools/src/events.rs` — ONLY allowed modification is adding `Paused` + `Aborted` variants (with `#[non_exhaustive]` preserving semver). No other edits.

**New workspace members to declare:**
- `crates/kay-cli` (exists as stub today; populated in Wave 7)

---

## Locked decisions (resolves all open questions)

### DL-1 — Parity fixture: REAL (not flagged)

**Question:** Does `forgecode-parity-baseline` tag exist? If not, skip parity diff.

**Decision:** Tag exists at `9985d77`. Wave 7 WILL ship a real parity fixture.

**Implementation:**
- Create `tests/fixtures/forgecode-banner.txt` — captured by running `git checkout forgecode-parity-baseline -- crates/forge_main/src/banner.rs` in a scratch worktree, extracting the pre-brand-swap strings, and storing as golden file
- Create `tests/fixtures/forgecode-prompt.txt` — same approach for prompt string
- T-5 parity diff test: invoke `kay` interactive mode, capture stdout, normalize brand tokens (`forge`→`kay`, `ForgeCode`→`Kay`), diff against fixture; ANY non-brand divergence = test failure
- Fixture-capture script lives at `scripts/capture-parity-fixtures.sh` (gitignored output dir, committed fixture files)

**Owner wave:** Wave 7 Tasks 11-12

**INFO-a from VALIDATION resolved.**

---

### DL-2 — Pause/Resume semantics: BUFFER-AND-REPLAY

**Question:** When the agent loop receives `ControlMsg::Pause`, what happens to in-flight model tokens and tool calls?

**Decision:** Buffer-and-replay semantics.

**Detailed behavior:**
| Event in loop | On Pause | On Resume |
| ------------- | -------- | --------- |
| Model stream TextDelta arrives | Buffered in VecDeque<AgentEvent> | Replayed in order |
| Tool call completes | Buffered | Replayed |
| Ctrl-C arrives (Abort) | Flush buffer as Aborted (discarded pending events not emitted, but logged at TRACE) | N/A |
| In-flight tool call | Runs to completion (2s grace window for cleanup); its ToolOutput buffered | Replayed |

**Wire emission:** `Paused` event emitted immediately on Pause receipt (before any buffering). `Resumed` NOT a separate variant — next downstream event (replayed or fresh) signals resumption. Avoids adding a 4th variant; keeps wire count at 13.

**Pause while paused:** No-op, emits no additional Paused event (idempotent).

**Owner wave:** Wave 4 Step 9

**INFO-b from VALIDATION resolved; Q2 from BRAINSTORM resolved.**

---

### DL-3 — forge_main retention: KEEP AS-IS THROUGH PHASE 10

**Question:** When does `forge_main` crate get deleted or rebranded internally?

**Decision:** `forge_main` crate retained as-is (no source edits) through Phase 10.

**Phase 5 behavior:**
- `forge_main` crate: untouched. Existing `forge` binary continues to build and ship.
- `kay-cli` crate: NEW, populated in Wave 7 with kay binary.
- Both binaries ship in v0.3.0 release (`forge` + `kay`); both point at the same agent loop internally.
- `kay-cli/src/main.rs` does NOT import from `forge_main`; it's parallel, not layered. (E8 was ambiguous on this — clarifying here: `kay-cli` is independent; `forge_main` is a legacy alias that continues to work.)

**Phase 10 behavior (future work, NOT Phase 5 scope):**
- `forge_main` internal modules rebranded file-by-file (editor.rs, input.rs, state.rs, completer/, etc.)
- `forge_main` crate renamed to `kay-cli-legacy` or deleted entirely
- `forge` binary deleted

**Rationale:** E8 wanted minimum churn in Phase 5. Parallel coexistence > migration. The rebrand-surface (banner/prompt/help/exit-codes) is port-only in Phase 5; internal rebrand is deferred work.

**Cargo.toml updates in Wave 7:**
- `crates/forge_main/Cargo.toml` description updated from "...will be rebranded to kay-cli in Phase 5" to "ForgeCode-compatibility binary; Phase 10 will rebrand or delete"
- `Cargo.toml` workspace members already include both crates

**Owner wave:** Wave 7 Task 7 (retention confirmation; zero code changes to forge_main source)

**INFO-c from VALIDATION resolved; Q3 from BRAINSTORM resolved.**

---

### DL-4 — Paused and Aborted variant additions: CONFIRMED

**Question:** Add 2 new AgentEvent variants or emit existing variants with special payloads?

**Decision:** Add both as new variants (variant count 11 → 13). Both `#[non_exhaustive]`-friendly since the parent enum is already `#[non_exhaustive]`.

**Variant shapes (LOCKED):**

```rust
// In crates/kay-tools/src/events.rs (added to existing enum)
#[non_exhaustive]
pub enum AgentEvent {
    // ... 11 existing variants unchanged ...

    /// Agent loop paused by ControlMsg::Pause. Emitted once per Pause receipt.
    /// Resume is signaled by the next downstream event (no separate Resumed variant).
    Paused,

    /// Agent loop aborted. Emitted exactly once per agent run that exits abnormally.
    /// Downstream consumers should treat this as a terminal event.
    Aborted {
        /// Machine-readable reason tag. One of:
        ///   "user_ctrl_c"              — SIGINT received
        ///   "max_turns"                — turn budget exhausted
        ///   "verifier_fail"            — verify gate declined task_complete
        ///   "sandbox_violation_prop"   — SandboxViolation terminated the turn (rare path)
        ///   "other:<free-form>"        — escape hatch; reason free-form after colon
        reason: String,
    },
}
```

**Wire shape (AgentEventWire mirror):** same, matched variant-for-variant.

**Insta snapshot count:** 16 (14 existing variants × 1 snapshot each + `Paused` + `Aborted{reason: "user_ctrl_c"}` = 16 — matches TEST-STRATEGY Wave 1 exit).

Wait — existing count is 11; adding 2 brings it to 13. Why does IMPL-OUTLINE say 16 snapshots? Because:
- 11 existing + 2 new = 13 variants
- `Aborted` has 4 canonical reason strings we snapshot (user_ctrl_c, max_turns, verifier_fail, sandbox_violation_prop) = 3 additional snapshots beyond the single-variant representative
- Total = 13 variant-representatives + 3 additional Aborted-reason snapshots = 16

**Contract doc:** `.planning/CONTRACT-AgentEvent.md` (Wave 1 Step 7) will enumerate all 13 variants + the reason-tag taxonomy.

**Owner wave:** Wave 1 Tasks 5-6

**Q4 from BRAINSTORM resolved.**

---

### DL-5 — events-buffer flag: DEFERRED TO CONDITIONAL TASK

**Question:** What's the shape of the `--events-buffer` flag?

**Decision:** Deferred. Not implemented in Phase 5 unless profiling reveals DoS.

**Rationale:** BRAINSTORM §Product-Lens risk #9 (JSONL stdout DoS) is speculative. The default mode is unbuffered, which is correct for real-time streams. Buffered mode adds complexity and would benefit from real usage data.

**Phase 5 behavior:**
- Default: unbuffered stdout (line-at-a-time flush)
- No `--events-buffer` flag in clap derive
- If DoS is observed during beta (Phase 9-10), add as `--events-buffer <N_lines>` flag with N=0 meaning unbuffered

**Placeholder task in Wave 7 exit criteria:** "Measure stdout throughput at 100 tool calls/turn — if write() blocks >10ms cumulative, file a Phase 6+ backlog item."

**Owner wave:** Wave 7 (measurement only; no flag in Phase 5)

**Q5 from BRAINSTORM resolved.**

---

### DL-6 — REQUIREMENTS.md traceability fix: IN-PHASE HOTFIX

**Question:** When does the CLI-04/05/07 traceability gap get fixed?

**Decision:** First commit of Wave 7 (before kay-cli implementation tasks) patches REQUIREMENTS.md lines 242-249 traceability table to add CLI-04, CLI-05, CLI-07 rows pointing to Phase 5. This is a single-line-per-REQ append, DCO-signed, separate commit from code changes.

**Owner wave:** Wave 7 pre-task commit

**WARN item from VALIDATION resolved.**

---

### DL-7 — ROADMAP.md Phase 4 checkbox: OUT-OF-PHASE HOTFIX

**Question:** When does ROADMAP.md get updated to mark Phase 4 `[x]` COMPLETE?

**Decision:** Immediately after this CONTEXT.md is written. Single-commit hotfix before Step 5 (analyze-dependencies).

**Owner:** immediately (side-task, single commit, DCO-signed).

---

## Plan-time parameters locked

These additional constraints inform the planner but are not "open questions":

| Parameter | Locked value | Source |
| --------- | ------------ | ------ |
| Branch name | `phase/05-agent-loop` | Already exists (pushed from origin/main @ 1ae2a7f) |
| Commit trailer | `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` | PROJECT.md non-negotiable DCO |
| Phase close tag | `v0.3.0` ED25519-signed | Silver bullet convention; E7 |
| CI workflow | `.github/workflows/ci.yml` matrix {macos-14, ubuntu-latest, windows-latest} | Inherited from Phase 4 (DCO loop fix) |
| Coverage tool | `cargo-llvm-cov` (already in workspace from Phase 4) | Inherited |
| Snapshot tool | `insta` (already in workspace) | Inherited |
| Property-test tool | `proptest` (already in workspace) | Inherited |
| E2E test tool | `assert_cmd` + `predicates` | NEW dev-dep (add to workspace in Wave 7) |
| compile-fail tool | `trybuild = "1"` | NEW dev-dep (add to workspace in Wave 6c early step) |
| AgentEvent variant count target | 13 | DL-4 |
| Insta snapshot count target | 16 | DL-4 |
| event_filter coverage threshold | 100% line + 100% branch | TEST-STRATEGY; SHIP BLOCK if missed |
| sage_query nesting depth cap | 2 | BRAINSTORM E5 |
| R-2 max_image_bytes default | 20 MiB (20 * 1024 * 1024) | BRAINSTORM §Product-Lens risk #4 |
| R-1 PTY tokenizer separators | `[\s;|&]` — whitespace, semicolon, pipe, ampersand | ROADMAP success criterion 6; BRAINSTORM E9 |
| YAML persona bundled names | `forge.yaml`, `sage.yaml`, `muse.yaml` | BRAINSTORM E3 |
| Exit codes | 0=success, 1=max_turns, 2=sandbox_violation, 3=config_error, 130=SIGINT | CLI-03; BRAINSTORM E6 |

---

## Scope-creep redirects

Any of the following surfacing during planning should be redirected to deferred-ideas, NOT Phase 5:

| Tempting addition | Redirect |
| ----------------- | -------- |
| Session persistence (resume after Ctrl-C restart) | Phase 6 (SESS-*) |
| Retrieval-augmented context | Phase 7 (CTX-*) |
| Real verifier (not NoOp) | Phase 8 (VERIFY-*) |
| Tauri GUI | Phase 9 |
| TUI | Phase 9.5 |
| `cargo install kay` distribution | Phase 10 |
| Internal forge_main rebrand (editor/state/completer) | Phase 10 |
| `--events-buffer` flag implementation | Deferred; Phase 6+ if DoS observed |
| More personas beyond forge/sage/muse | Phase 11+ via Persona::from_path (already supported) |
| Richer Aborted reason taxonomy | Out-of-scope; 5 reasons suffice for v0.3.0 |

---

## Next steps (for downstream agents)

1. **`gsd-analyze-dependencies`** (Step 5) → `05-DEPENDENCIES.md` — map Phase 5's dependence on Phases 2/3/4 + new intra-phase wave DAG
2. **`gsd-plan-phase`** (Step 6) → `05-PLAN.md` — convert 7-wave IMPL-OUTLINE into atomic tasks with exact file-paths, test names, and commit messages; honor all DL-1 through DL-7 locked decisions; include DL-6 REQUIREMENTS.md traceability fix as the first commit of Wave 7

---

**Next step:** Step 5 `gsd-analyze-dependencies` → `05-DEPENDENCIES.md`.
