# Milestone 13: Feature Parity with Forge

**Status:** IN PROGRESS
**Created:** 2026-04-26
**Target:** Kay meets or exceeds Forge capabilities

## Executive Summary

Kay currently has 22 identified feature gaps compared to Forge. This milestone aims to close all critical and high-priority gaps, with medium/low priority addressed as feasible.

## Gap Categories

| Priority | Count | Description |
|----------|-------|-------------|
| CRITICAL | 3 | Tool execution, Agent loop, Planning |
| HIGH | 4 | Review, Task delegation, Test, Build commands |
| MEDIUM | 8 | History, Session CLI, Markdown, etc. |
| LOW | 5 | Help, Embedding, Diff, etc. |
| Provider | 2 | More providers, Transforms |

## Critical Gaps

### C1: Tool Call Execution Infrastructure
**Current:** Kay cannot execute tool calls
**Target:** Implement ToolExecutor that handles all tool operations
**Files to create/modify:**
- `crates/kay-tools/src/executor.rs` (new)
- `crates/kay-core/src/lib.rs` (export)
- `crates/kay-core/src/agent.rs` (integration)

**Tasks:**
1. Create `ToolExecutor` struct with execution methods
2. Implement `execute_read(path, line_range)` → content
3. Implement `execute_write(path, content, overwrite)`
4. Implement `execute_patch(path, old, new)`
5. Implement `execute_search(query, options)` → results
6. Implement `execute_fetch(url, options)` → response
7. Implement `execute_shell(cmd, options)` → output
8. Add test coverage

**Verification:** Unit tests for each executor method

### C2: Agent Loop / Turn Orchestration
**Current:** Single-prompt response only
**Target:** Full agent loop with multi-turn conversation
**Files to create/modify:**
- `crates/kay-core/src/agent_loop.rs` (new)
- `crates/kay-core/src/agent.rs` (enhance)
- `crates/kay-cli/src/interactive.rs` (wire)

**Tasks:**
1. Create `AgentLoop` struct
2. Implement conversation state machine
3. Implement tool call → execution → response cycle
4. Implement stop condition detection
5. Implement max_turns limit
6. Wire into CLI

**Verification:** Integration test with multi-turn conversation

### C3: Planning System Enhancement
**Current:** Basic task breakdown
**Target:** Full Forge-style planning with requirements
**Files to create/modify:**
- `crates/kay-core/src/planner.rs` (enhance)

**Tasks:**
1. Add REQ-ID requirement tracking
2. Add threat model generation per phase
3. Add rollback plan generation
4. Add quality gate compliance (9 dimensions)
5. Add milestone tracking

**Verification:** Create plan with all features and verify structure

## High Priority Gaps

### H1: Build/Check/Fmt/Clippy Commands
**Files to modify:**
- `crates/kay-cli/src/main.rs`
- `crates/kay-cli/src/build.rs` (new)

**Tasks:**
1. Create `kay build` command (cargo build --workspace)
2. Create `kay check` command (cargo check --workspace)
3. Create `kay fmt` command (cargo fmt -p <crate>)
4. Create `kay clippy` command (cargo clippy)
5. Add error handling and progress display

**Verification:** All commands work on vs-others codebase

### H2: Test Execution Command
**Files to modify:**
- `crates/kay-cli/src/main.rs`
- `crates/kay-cli/src/test_cmd.rs` (new)

**Tasks:**
1. Create `kay test` command
2. Test suite discovery (find crates/*/tests/*.rs)
3. Test execution with timeout
4. Result parsing and display
5. Coverage reporting

**Verification:** `kay test` runs and reports results

### H3: Code Review Workflow
**Files to create/modify:**
- `crates/kay-cli/src/review.rs` (new)
- `crates/kay-cli/src/main.rs`

**Tasks:**
1. Create `kay review` command
2. Linting compliance checking (cargo clippy)
3. Architecture pattern validation
4. Report generation

**Verification:** `kay review` produces review report

### H4: Task Delegation
**Files to create/modify:**
- `crates/kay-tools/src/task.rs` (new)
- `crates/kay-core/src/lib.rs`

**Tasks:**
1. Create `Task::spawn()` for agent delegation
2. Implement session continuation
3. Implement parallel execution
4. Implement result aggregation

**Verification:** Spawn sub-agent and receive result

## Medium Priority Gaps

### M1: Command History (REPL)
**Files to modify:**
- `crates/kay-cli/src/interactive.rs`

**Tasks:**
1. Add readline/history crate
2. Implement history tracking
3. Arrow key navigation
4. History search (Ctrl+R)

### M2: Session Management CLI
**Files to modify:**
- `crates/kay-cli/src/main.rs`
- `crates/kay-cli/src/session_cmd.rs` (new)

**Tasks:**
1. `kay session list`
2. `kay session load <id>`
3. `kay session delete <id>`

### M3: Markdown Enhancements
**Files to modify:**
- `crates/kay-cli/src/markdown.rs`

**Tasks:**
1. Add italic support (single asterisk)
2. Add code blocks (triple backtick)
3. Add table rendering
4. Add link rendering
5. Progressive display

### M4: Spinner / Progress
**Files to create:**
- `crates/kay-cli/src/spinner.rs`

**Tasks:**
1. Create Spinner struct
2. Progress animation
3. Custom characters
4. Async support

### M5: JSON Repair
**Files to create:**
- `crates/kay-json-repair/` (new crate)

**Tasks:**
1. Malformed JSON recovery
2. Partial JSON parsing
3. Strict mode option

### M6: Retry Logic
**Files to create:**
- `crates/kay-core/src/retry.rs`

**Tasks:**
1. Automatic retry decorator
2. Exponential backoff
3. Max retry limits

### M7: Token Budget Management
**Files to modify:**
- `crates/kay-context/src/compress.rs`
- `crates/kay-session/src/history.rs`

**Tasks:**
1. Token counting (approximate)
2. Budget enforcement
3. Auto-compaction trigger

### M8: Template Engine
**Files to create:**
- `crates/kay-template/` (new crate)

**Tasks:**
1. Template rendering
2. Variable substitution
3. Conditional rendering

## Low Priority Gaps

### L1: Help System
**Tasks:** `kay --help` and command-specific help

### L2: Diff Highlighting
**Tasks:** Diff visualization for code changes

## Execution Plan

### Phase 1: Core Infrastructure (Critical)
1. Tool Call Execution Infrastructure
2. Agent Loop / Turn Orchestration
3. Build/Check/Fmt/Clippy Commands

### Phase 2: Planning & Review
4. Planning System Enhancement
5. Test Execution Command
6. Code Review Workflow

### Phase 3: Delegation & Sessions
7. Task Delegation
8. Session Management CLI

### Phase 4: UX Enhancements
9. Command History
10. Markdown Enhancements
11. Spinner / Progress
12. JSON Repair
13. Retry Logic
14. Token Budget
15. Template Engine

### Phase 5: Polish
16. Help System
17. Diff Highlighting

## Verification

Each gap will be verified by:
1. Unit tests for the feature
2. Integration test demonstrating the feature
3. Live testing in vs-others codebase

## Sign-off

- Milestone: M13
- Status: IN PROGRESS
- Created: 2026-04-26
