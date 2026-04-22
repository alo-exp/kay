# Phase 7: Context Engine — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 07-context-engine
**Mode:** autonomous (§10e) — no interactive questions; all decisions resolved from upstream artifacts
**Areas discussed:** ContextEngine wiring, Embedding strategy, Token counting, rusqlite promotion, FileWatcher backend

---

## ContextEngine wiring into run_turn

| Option | Description | Selected |
|--------|-------------|----------|
| Add fields to RunTurnArgs | Add `context_engine`, `context_budget`, `initial_prompt`; call retrieve() at top of run_turn; real injection Phase 8+ | ✓ |
| Separate channel | Pass ContextPacket via a new mpsc channel alongside model_rx | |
| Post-turn injection | Inject context at the start of the next turn (lag by one) | |

**Decision:** Add 3 fields to `RunTurnArgs`; call `retrieve()` at top of `run_turn` before event loop. `ContextPacket` is assembled but not yet wired to the OpenRouter request (Phase 8+ concern). `run_turn` is an event consumer loop, not a prompt assembler.

---

## Embedding strategy for sqlite-vec

| Option | Description | Selected |
|--------|-------------|----------|
| NoOpEmbedder default | No network; ANN disabled; FTS5 only; FakeEmbedder for tests | ✓ |
| OpenRouter embeddings at index time | Real embeddings but adds network cost + latency to indexing | |
| Local ONNX embedding | Local model but adds 100MB+ binary weight + ONNX runtime dep | |

**Decision:** `NoOpEmbedder` default. `FakeEmbedder` (zero-vector, `#[cfg(test)]`) for deterministic W-4 tests. OpenRouter embedder deferred to Phase 8+.

---

## Token counting strategy

| Option | Description | Selected |
|--------|-------------|----------|
| chars().count() / 4 heuristic | Fast, portable, within 20% for code | ✓ |
| tiktoken-rs | Accurate BPE counting but requires C++ build toolchain | |
| byte_length / 4 | Simple but wrong for multibyte UTF-8 | |

**Decision:** `chars().count() / 4`. Use `.chars()` not `.len()` to handle non-ASCII correctly. Budget has 12.5% reserve to absorb estimation error.

---

## rusqlite workspace promotion

| Option | Description | Selected |
|--------|-------------|----------|
| Promote to [workspace.dependencies] | Single source of truth; both kay-session and kay-context use workspace = true | ✓ |
| Keep crate-local, duplicate in kay-context | Two places to update on version bump | |

**Decision:** Promote to workspace. Only change to `kay-session/Cargo.toml` is `rusqlite = { workspace = true }`.

---

## FileWatcher backend and ignore patterns

| Option | Description | Selected |
|--------|-------------|----------|
| notify 6.1 + notify-debouncer-mini 0.4, 500ms | Proven, cross-platform, minimal deps | ✓ |
| inotify/kqueue directly | Platform-specific, no debounce | |
| Poll-on-access (no watcher) | Simple but misses changes between queries | |

**Decision:** notify 6.1 + notify-debouncer-mini 0.4. 500ms debounce. Ignore `*.lock`, `target/`, `.git/`, `*.tmp`, `*.swp`. Watch `.rs`, `.ts`, `.tsx`, `.py`, `.go`.

---

## Claude's Discretion

- Internal struct field ordering
- Whether FakeEmbedder uses zero-vectors or hash-based vectors (zero-vector simpler)
- KayContextEngine stub shape (empty struct acceptable)
- `<context>` block serialization format (Phase 8+ decision)

## Deferred Ideas

- OpenRouterEmbedder (Phase 8+)
- System prompt injection plumbing (Phase 8+)
- tiktoken-rs accurate counting (post-v1)
- sqlite-vec stable version unpin (once released)
