# Pitfalls Research — Kay

**Domain:** Open-source agentic coding agent (Rust harness + Tauri desktop UI) forked from ForgeCode, targeting Terminal-Bench 2.0
**Researched:** 2026-04-19
**Confidence:** HIGH (Tauri, PTY, OpenRouter, claw-code/leak, TB 2.0 verified with current sources) / MEDIUM (ForgeCode internals, KIRA ratios — training-data + project context only)

This document catalogs what will kill Kay before v1 ships — specific to forking ForgeCode, porting KIRA techniques, shipping Tauri across three OSes, and submitting to Terminal-Bench 2.0. Generic advice ("write tests") is deliberately omitted.

---

## Critical Pitfalls

### Pitfall 1: Benchmark Overfitting to Terminal-Bench 2.0's 89-Task Shape

**What goes wrong:**
Kay tops 81.8% on the local Harbor run but scores materially lower on the official submission, or scores 82% on TB 2.0 and is useless on a real codebase. The 89 tasks are narrow (model training, sysadmin, scripting) and run in Daytona containers under a binary pass/fail score — easy to tune a harness for Docker+pytest shape that doesn't translate to "help me fix this React bug."

**Why it happens:**
- Agents get quietly tuned against oracle `solution.sh` scripts they've seen during development ("solution leakage").
- Prompt-level heuristics that pattern-match TB's instruction phrasing ("Your task is to…") don't help real users whose prompts are messier.
- Retry-until-pass loops exploit TB's determinism — real users don't let an agent run `pytest` 40 times.
- Context engines that index by file-type (because TB tasks are small, tight repos) fail on 500kloc monorepos.

**How to avoid:**
- Treat `tasks/` in the TB corpus as a test set — never train, tune prompts, or design heuristics from it. Hold out a subset and never look at it until submission.
- Keep a **parallel eval set** of real repos (at minimum: a Rails app, a React+TS frontend, a Rust crate, a Python package, a monorepo with >10k files). Require Kay to perform on both before shipping.
- Cap the agent's retry budget in the harness — ForgeCode runs at a specific `turn_limit`/`time_limit`; matching the official submission settings locally is mandatory.
- Audit any prompt or heuristic that name-checks "pytest", "tests/", "solution.sh", or Docker idioms.

**Warning signs:**
- Local score climbs faster than performance on ad-hoc real repos.
- Regressions when the model changes by a point release (tightly-tuned prompts break).
- You find yourself writing task-specific branches in the harness ("if file named `train.py` then…").

**Phase to address:** Phase 2 (harness foundations) — set the red lines before any eval runs. Phase 5 (submission prep) verifies.

---

### Pitfall 2: ForgeCode License and NOTICE Hygiene Failure

**What goes wrong:**
Kay ships v1 with broken NOTICE preservation, missing attribution, or the ForgeCode team files a friendly-but-firm attribution complaint that forces a re-tag. Worse: a downstream packager (Homebrew, AUR) flags the repo and it gets pulled.

**Why it happens:**
- Apache-2.0 §4 (a/b/c/d) is precise: retain the original NOTICE file, provide copies of the license, state "what changes you made," don't imply endorsement by the original authors.
- Forks routinely strip `NOTICE` because the contents feel "not about our project."
- IDE refactors that rename modules or repackage crates lose file-level copyright headers silently.
- The project README trumpets "Kay beats ForgeCode" without a clear "derived from ForgeCode" statement — triggers trademark/endorsement concerns.

**How to avoid:**
- Day-1 checklist: preserve `NOTICE` verbatim, append (not replace) Kay-specific entries, keep every `Copyright (c)` header in every file, maintain an `ATTRIBUTIONS.md` that lists upstream commits the fork was based on.
- Use `cargo-about` or `cargo-deny license` in CI to fail builds if any dependency license changes.
- Repo-top banner: "Kay is a fork of [ForgeCode](…), Apache-2.0 licensed. See `NOTICE` and `ATTRIBUTIONS.md`."
- Avoid wording like "endorsed by," "compatible with ForgeCode's team," or using their logos.

**Warning signs:**
- PRs that "clean up" NOTICE or "simplify attributions."
- Dependency updates that flip a transitive crate to GPL/AGPL (common on crypto libs, some parsers).
- README copy drifts from "derived from" to "inspired by" (deliberate distancing is a red flag).

**Phase to address:** Phase 1 (fork + attribution) — baseline set before any code changes. CI gate enforced from Phase 1 onward.

---

### Pitfall 3: ForgeCode Over-Specialization Baggage

**What goes wrong:**
Kay inherits prompts, schemas, context-engine heuristics, and CLI UX that were tuned against ForgeCode's benchmark runs — including tricks that only make sense for TB-shaped tasks. When Kay tries to serve interactive users through the Tauri UI, those behaviors feel wrong (overly terse, assumes `/tmp` is writable, assumes a single fresh Docker env, etc.).

**Why it happens:**
- Benchmark-tuned harnesses bake in assumptions: "current working directory is always the task root," "there are never external side effects," "the user never interrupts." None hold in a desktop UI.
- Schema hardening for TB tasks (aggressive truncation reminders, field reordering) can backfire on long interactive sessions where context is naturally long.
- System prompts often contain TB-specific phrases ("run the tests to verify," "solution.sh").
- Context engine is indexed for small, self-contained repos; interactive users open monorepos.

**How to avoid:**
- Upon fork, audit every string literal in system prompts, every default config, every hard-coded path. Build a `KAY_CHANGES.md` listing intentional divergences.
- Separate "benchmark mode" and "interactive mode" prompt sets from day one — don't try to serve both with one prompt. Benchmark mode stays close to ForgeCode; interactive mode is Kay's reinvention.
- Add a "fresh user" soak test: first day of Tauri use on a real repo; watch logs for TB-specific behaviors (asking to create `tests/`, assuming `pytest`).
- Keep a list of "ForgeCode-isms" to rewrite before v1 (one PR per ism, reversible if they hurt benchmark score).

**Warning signs:**
- Agent says "run the tests" on repos with no tests.
- Agent suggests `solution.sh` or creates it unprompted.
- Context engine picks the wrong files because it's scoring by "proximity to task-root" rather than relevance.
- Interactive users report the agent feels "robotic" compared to Claude Code.

**Phase to address:** Phase 2 (harness foundations) — identify the isms; Phase 4 (UI integration) — rewrite for interactive use.

---

### Pitfall 4: Native Tool Calling Breaks on Non-Spec JSON

**What goes wrong:**
A model (especially on OpenRouter, which proxies many providers) emits `arguments: null`, `arguments: "{}"` (string, not object), or includes a stray `tool_use` chunk with a non-ASCII `id`. Kay's tool dispatch panics, the user sees a stack trace, benchmark runs fail mid-task.

**Why it happens:**
- OpenAI's tool-calling schema is the *de facto* standard, but every provider implements a slightly different dialect. OpenRouter's own telemetry (Aug 2025+) flagged that "providers omit the `arguments` field for tools with no parameters" and that Gemini often returns `finish_reason: 'stop'` when tool calls are pending alongside encrypted reasoning.
- Streaming makes this worse: tool-call deltas arrive in fragments, with the JSON assembled client-side. A missing closing brace mid-stream turns into `serde_json::Error`.
- Models are stochastic — a weird JSON blob can appear once per 1,000 calls, meaning local tests pass but nightly eval runs flake.

**How to avoid:**
- Implement a two-pass JSON tolerant parser: (1) strict `serde_json`, (2) on failure, coerce `""`/`null`/`"{}"` → `{}`, auto-close unterminated strings, then retry.
- Never `.unwrap()` on tool-call parsing — return a structured `ToolCallError` that the orchestration loop can retry with a "please re-emit the tool call in valid JSON" message.
- Per OpenRouter's "Exacto" announcement: allow the user to opt into Exacto-routed endpoints for production runs; use standard routing for dev.
- Log every raw tool-call payload to a ring buffer so when users file bugs, you have ground truth.

**Warning signs:**
- Flaky benchmark runs whose failures cluster on specific providers.
- Telemetry shows occasional `serde_json::Error::EOF` in tool parsing.
- Users on Gemini or DeepSeek report the agent "freezes" mid-task (`finish_reason: 'stop'` with pending tool calls).

**Phase to address:** Phase 2 (harness foundations) — tolerant parser is table stakes before any eval work.

---

### Pitfall 5: Marker-Based Command Completion Race with User Input

**What goes wrong:**
Kay appends `__CMDEND__<seq>__` sentinel after each shell command, then polls for it. A user in the Tauri UI types `echo "__CMDEND__99__"` (curious, malicious, or pasting logs back). The poller sees the fake marker, concludes the command finished, misses the real command's output, and subsequent agent reasoning works from truncated state.

**Why it happens:**
- Sentinel markers assume the output stream is produced only by the command, not by interactive input or concurrent processes.
- Desktop UI allows users to inject keystrokes into the agent's terminal (feature: watch and steer; liability: violate the marker's invariant).
- Long-running commands (`npm install`, test suites) produce enough output that even a random UUID collision becomes non-zero over millions of sessions.

**How to avoid:**
- Use **cryptographically random markers** per command (e.g., 128-bit UUID encoded in a form unlikely in user input), and reject any command whose *input* contains the marker substring before execution.
- Separate the agent's PTY from the user-visible terminal: the user sees a read-only mirror; to inject commands, they use a dedicated "user command" lane the agent explicitly polls.
- Track markers by `(sentinel, exit_code)` where `exit_code` is captured via `$?` immediately after the sentinel — validates authenticity.
- Document the boundary in the UI: "typing here pauses the agent" vs. "typing here sends a message."

**Warning signs:**
- Bug reports of "agent thinks my command finished but it didn't."
- Sessions where the log shows two sentinels with the same sequence number.
- Tests show race conditions when user input is injected during long commands.

**Phase to address:** Phase 3 (KIRA techniques) — design the marker scheme before implementation. Phase 4 (Tauri UI) — enforce the input-lane boundary.

---

### Pitfall 6: Multi-Perspective Verification Exploding Token Costs

**What goes wrong:**
KIRA's "test engineer + QA engineer + end-user critic" verification trio triples token usage per completion. Kay's benchmark runs start costing $50-100 each, users' OpenRouter bills spike, and the economic pitch ("beat ForgeCode on an open model") evaporates.

**Why it happens:**
- Verification passes are seductive: they're measurably effective on hard tasks, so the instinct is to always run them.
- On easy tasks (90% of real work), verification is pure overhead — the first answer was right.
- Benchmark bias: TB 2.0 rewards correctness, so verification pays off; real interactive use rewards responsiveness, where it doesn't.

**How to avoid:**
- **Gate verification on a confidence/complexity signal**: only invoke the trio when the primary agent's self-score is low, the task touched >N files, or tests failed. "Fire if risky" not "fire always."
- Make the number of critics configurable; default to 1 for interactive, 3 for benchmark.
- Measure cost-per-task in CI: a regression that adds 30% tokens fails the build.
- Expose cost projection in the Tauri UI ("This task will cost ~$0.14 at current rates") so users catch runaway spending in real time.

**Warning signs:**
- Eval runs exceed the budget set in the roadmap.
- Users complain about OpenRouter bills after the first week.
- Telemetry shows the primary answer and the verified answer agree >90% of the time (verification isn't changing outcomes).

**Phase to address:** Phase 3 (KIRA techniques) — design the gating from the start, not retrofitted.

---

### Pitfall 7: Multimodal `image_read` Prompt-Token Bloat

**What goes wrong:**
Every terminal screenshot pumps 1-2k tokens into the context. Fire it once per command and a 50-turn session burns 50-100k tokens of image data that could have been textual. Context window exhaustion mid-task, or the first few turns get evicted when summarization kicks in.

**Why it happens:**
- Screenshots feel safer than text parsing — they show the "truth" of the terminal — so developers fire them reflexively.
- Base64 image encoding is expensive in tokens (roughly 85 tokens per 512×512 tile for Anthropic/OpenAI vision models).
- There's no natural off-switch: if the harness always does `capture_screen() -> image_read()`, it always pays.

**How to avoid:**
- **Trigger `image_read` only when text parsing fails**: if the agent's text-based polling got a clean sentinel + exit code, don't screenshot. Fire it only when output contains ANSI escape soup, TUI grids (ncurses apps), or garbled bytes.
- Cap images per turn (1-2) and per session (10-20). Enforce in the tool wrapper, not as a prompt suggestion.
- Downsample before encoding — a 512×256 tile is adequate for most terminal states.
- Mark images as ephemeral in the context strategy: evict older images first when summarizing.

**Warning signs:**
- Sessions that hit context window limits earlier than expected.
- Context-summarization events occurring early in sessions with heavy terminal output.
- Benchmark token usage flat or worse than baseline despite KIRA techniques.

**Phase to address:** Phase 3 (KIRA techniques) — design the firing policy before enabling the tool.

---

### Pitfall 8: Tauri Code Signing and Notarization Blocking Releases

**What goes wrong:**
v1 ships, macOS users see "Kay.app can't be opened because Apple cannot check it for malicious software," Windows users see SmartScreen red flags, Linux users grumble about unsigned AppImages. The project's credibility takes a hit on day one, and fixing it requires dev accounts, hardware tokens, and multi-day Apple turnaround.

**Why it happens:**
- Apple Developer ID: $99/year, needs a paid account (free accounts cannot notarize).
- Notarization is a separate step after signing; notarization stalls ("stuck at notarizing the Mac app") are common and can last hours, blocking CI.
- Windows Authenticode certificates increasingly require EV certificates stored on physical USB tokens or cloud HSMs — automation is awkward.
- Sidecars (`externalBin`) in Tauri 2 specifically break notarization — if Kay bundles the ForgeCode binary as a sidecar, it hits this bug.
- Linux has no equivalent signing story; AppImage signing with GPG is informal at best.
- Auto-update infra (`tauri-plugin-updater`) requires a signed manifest and a hosted server — additional moving parts.

**How to avoid:**
- Apply for Apple Developer ID and Azure Code Signing (or equivalent Windows cert) in **Phase 1**, not Phase 5 — onboarding takes weeks.
- Prefer embedding dependencies in the main Rust binary over `externalBin` sidecars (avoid the notarization bug entirely).
- Set CI notarization timeouts to 2 hours with retry-on-Apple-queue logic; surface the "stuck" state as a red build, not a silent hang.
- Ship a signed+notarized dev build on PR merge to main, not only on release. Problems surface earlier.
- For Linux, commit to one format (AppImage or `.deb`+`.rpm`) — don't try to sign all three formats in v1.
- Auto-update: use a single GitHub-Release-based updater for v1; defer CDN/custom server to v2.

**Warning signs:**
- CI failing intermittently on notarization ("net timeout reaching Apple's notary service").
- Users reporting SmartScreen warnings days after release (meant the cert's reputation hasn't built up).
- Any sidecar in `tauri.conf.json` — immediate yellow flag.
- "We'll handle signing at the end" on the roadmap.

**Phase to address:** Phase 1 (infra + accounts); Phase 5 (release) verifies end-to-end.

---

### Pitfall 9: Tauri Event-Emit Memory Leak on Long Sessions

**What goes wrong:**
Kay's Tauri UI streams the agent's trace via `emit` events. In a 2-hour session with thousands of events, the renderer process balloons to 1-2 GB, and eventually crashes. Users report "Kay got sluggish after lunch."

**Why it happens:**
- Documented Tauri 2 bug (issues #12724, #13133, #852): emitting events from Rust to JS leaks memory in the webview because `transformCallback` registers a callback on `window` that's never freed.
- Channel-based emission has the same issue — UUIDs accumulate.
- Long sessions with high-frequency events (token-by-token streaming) are exactly the worst-case shape.

**How to avoid:**
- Prefer **Tauri channels** for streaming tokens with explicit channel lifecycle management; close channels on session end.
- Batch low-priority events (tool-call telemetry, cost updates) at 100ms intervals instead of firing per-token.
- For the highest-frequency stream (agent reasoning tokens), consider a local WebSocket or a shared-memory ring buffer rather than IPC — bypass the leak entirely.
- Run a weekly canary: launch the app, run a 4-hour scripted session, measure RSS at start and end. Regression test.
- Subscribe to Tauri GitHub issues on IPC memory — upstream fixes land intermittently.

**Warning signs:**
- Memory Activity/Task Manager showing webview process steadily growing past ~500 MB.
- Users reporting the UI getting sluggish after extended use.
- Dev tools "Performance" tab showing retained listeners count climbing.

**Phase to address:** Phase 4 (Tauri UI) — pick the streaming primitive before building the session view.

---

### Pitfall 10: OpenRouter Tool-Calling Coverage Gaps

**What goes wrong:**
Kay defaults to a cheap/fast model for development (e.g., some DeepSeek or Qwen variant on OpenRouter), which doesn't support native tool calling or supports it badly. The harness silently degrades to ICL-style parsing that was supposed to be eliminated. Benchmark runs on GPT-5.4 succeed; user runs on "DeepSeek-Coder" fail mysteriously.

**Why it happens:**
- OpenRouter's catalog is massive (300+ models), but tool-calling quality varies sharply. OpenRouter themselves flagged this in their "Exacto" announcement (Aug 2025+).
- Some providers support tools but not *streaming + tools* simultaneously.
- Some return tool calls in a non-OpenAI format that OpenRouter attempts to re-emit in OpenAI shape with lossy fidelity.
- Error messages from OpenRouter are terse; the underlying provider's error is often swallowed.

**How to avoid:**
- Maintain an explicit allowlist of models Kay officially supports. Tier-1: Claude Opus/Sonnet, GPT-5/GPT-5-mini, Gemini 2.5 Pro. Tier-2: models Kay has verified on a tool-calling smoke test.
- On session start, if the selected model isn't on the allowlist, show a "Compatibility unknown" warning in the UI.
- Run a weekly tool-calling smoke test against every allowlisted model via CI; fail-open when regressions detected, surface in release notes.
- Prefer OpenRouter's `/exacto` endpoints for benchmark submissions (OpenRouter's own routing to providers with better tool-call accuracy).
- Never silently fall back to ICL parsing — if native tools are claimed and fail, raise a user-visible error.

**Warning signs:**
- Unexpected "tool call ignored" behavior that only reproduces on one model.
- Weekly tool-calling smoke test fails with no code change (upstream provider regressed).
- Users filing "Kay doesn't work" bugs that all share a non-allowlisted model.

**Phase to address:** Phase 2 (provider integration) — allowlist + smoke test from day one.

---

### Pitfall 11: Apache-2.0 CLA Contributor Adoption Friction

**What goes wrong:**
Kay's CLA requirement drives away drive-by contributors. PRs that would be 10-line bug fixes die when the contributor sees a legal form. The OSS buzz evaporates, and Kay starts to look like "a product disguised as open source" — the exact criticism that dogs Apache-style CLAs.

**Why it happens:**
- CLAs create an asymmetry: contributors grant broad rights; the project has relicensing optionality they don't.
- First-time contributors, especially students and hobbyists, treat legal forms as scary.
- Even with CLA-assistant bots (EasyCLA, cla-bot), the friction is visible and non-trivial.
- Many modern OSS projects have moved to DCO (Developer Certificate of Origin) with a simple `git commit -s` signoff — contributors perceive this as far lighter.

**How to avoid:**
- **Seriously consider DCO instead of CLA.** DCO gets you 90% of the provenance claim with 10% of the friction. Only adopt CLA if there's a concrete future plan that requires it (relicensing, a paid commercial variant).
- If CLA is kept: integrate EasyCLA or cla-assistant so signing is in-PR and fast; accept GitHub identity instead of full legal name for small contributions; allow contributor-signing-by-email for corporate blockers.
- Be transparent in `CONTRIBUTING.md` why CLA exists ("future proofing for possible dual-licensing") — avoid the appearance of a land grab.
- Track contributor retention: `contributors who opened 1 PR` vs `contributors who opened ≥2`. A big gap suggests CLA friction.

**Warning signs:**
- PRs opened, then abandoned, with no code review engagement.
- Issues like "I'd like to contribute but the CLA…"
- Fork activity disproportionate to contribution activity (people work around rather than with the project).

**Phase to address:** Phase 1 (governance) — pick CLA vs DCO before contribution volume grows.

---

### Pitfall 12: DMCA Exposure from Claude Code Leak Proximity

**What goes wrong:**
Kay, while Apache-2.0-clean on paper, borrows a phrase, a prompt, or a harness pattern that came from the March 2026 Claude Code leak. Anthropic files a DMCA. GitHub disables the repo for 10 days during adjudication. Kay's credibility and momentum evaporate.

**Why it happens:**
- The leak spread widely; developers who saw Claude Code's harness will have it in their heads.
- Claw-code's clean-room claim is legally unsettled — copying *its* code inherits whatever legal ambiguity it carries.
- "Inspiration" and "copying" live on a spectrum; a well-phrased system prompt that happens to match Claude Code's verbatim is hard to defend.
- Anthropic's first DMCA wave hit 8,100 repos simultaneously (per Prism News / 36Kr) before being partially retracted — collateral damage is real.

**How to avoid:**
- **Hard rule**: no contributor reads the leaked source, period. CONTRIBUTING.md spells this out; CLA/DCO attests it.
- No Kay code may be copied from claw-code or any derivative. Reference ForgeCode and KIRA's *public* writeups only. Treat claw-code as "known tainted."
- Maintain a clean-room log: for any pattern or prompt that resembles Claude Code, record how it was derived (public paper, ForgeCode source, independent invention).
- If a contributor admits leak exposure, do not accept their PRs for that area — keep their provenance clean.
- Have a DMCA response plan: pre-drafted counter-notice template, clean-room evidence repo, legal contact.

**Warning signs:**
- PRs with suspiciously polished prompts that match Claude Code's style.
- Contributors who mention "I saw in the leak that…"
- Commits that refactor an area beyond the stated issue's scope — could indicate copy-paste from an external source.

**Phase to address:** Phase 1 (governance + contribution policy). Active monitoring every phase.

---

### Pitfall 13: Expensive Benchmark Runs Burning OpenRouter Credits

**What goes wrong:**
Each full TB 2.0 run = 89 tasks × avg ~50k tokens × verification trio × 5 runs for submission average. At Claude Opus 4.6 prices, a single full submission run is $200-500. A maintainer accidentally runs nightly and burns $3-10k/month.

**Why it happens:**
- "Just one more run" creeps into daily dev.
- Failures on task N+1 trigger a full re-run rather than a targeted replay.
- Verification passes (Pitfall 6) multiply cost.
- No global cost cap in the harness.

**How to avoid:**
- Implement a **budget cap in the harness itself**: `--max-usd=50`, halts gracefully at limit.
- Add a "replay single task" mode that reuses cached logs for passing tasks.
- Separate `cheap` and `release` model configs: develop against a cheap model (GPT-5-mini, Haiku), validate on Opus/Sonnet only before submission.
- Treat benchmark runs as a PR-gated action — no nightly on main; run on green PRs or explicit dispatch.
- Log every run's cost to a shared dashboard; visible accountability.

**Warning signs:**
- OpenRouter spend graph has sawtooth spikes not tied to releases.
- Maintainer Slack/Discord has "oops I forgot…" messages.
- Per-task cost variance high — some tasks burning disproportionate tokens.

**Phase to address:** Phase 2 (harness) — cost cap built in. Phase 5 (submission) — replay and caching.

---

### Pitfall 14: Cross-Platform PTY + Windows ConPTY Surprises

**What goes wrong:**
Kay works perfectly on macOS + Linux during development; on Windows, command output is garbled with raw ANSI escapes, `npm install` hangs, the terminal doesn't resize when the Tauri window does, and `Ctrl+C` doesn't interrupt the child process.

**Why it happens:**
- Windows ConPTY is a genuinely different API (`CreatePseudoConsole`, named pipes) — `portable-pty` and `winpty-rs` abstract it but don't hide all the pain.
- Modern ConPTY flags (`PSEUDOCONSOLE_RESIZE_QUIRK`, `PSEUDOCONSOLE_WIN32_INPUT_MODE`, `PSEUDOCONSOLE_PASSTHROUGH_MODE`) aren't set by default in upstream `portable-pty` — output looks weird without them.
- Unix signals (`SIGINT`, `SIGTERM`) don't map cleanly to Windows; `Ctrl+C` sends a console control event that must be handled separately.
- Path separators, drive letters, and case-insensitive filesystem on Windows break assumptions in ForgeCode's (originally Unix-tested) harness.
- Terminal size detection (`tput cols`, `$COLUMNS`) returns different values inside ConPTY than in a real console.

**How to avoid:**
- Adopt `portable-pty` (wezterm's implementation, actively maintained) for the baseline, and layer a Kay-specific fork that passes the modern ConPTY flags. Document the divergence.
- Use `std::path::Path` everywhere; forbid string concatenation for paths. CI lint.
- Handle interruption via an abstraction layer: Unix → `kill(SIGTERM)`, Windows → `GenerateConsoleCtrlEvent`.
- Run the full test suite on Windows in CI — not just `cargo test`, the interactive PTY tests. GitHub Actions' `windows-latest` runner supports ConPTY.
- Budget time to test against Windows 10 *and* 11 (ConPTY behavior differs).

**Warning signs:**
- Tests skipped on Windows in CI.
- Users on Windows reporting "the terminal looks weird" — ANSI soup is the tell.
- `Ctrl+C` not killing long-running commands.
- Resize events not propagating (`npm install` progress bar shows at wrong width).

**Phase to address:** Phase 2 (harness) — PTY abstraction from the start. Phase 5 (cross-platform validation) — full Windows soak.

---

### Pitfall 15: Scope Creep Driven by "One More Wedge"

**What goes wrong:**
v1 was "ForgeCode parity + KIRA + Tauri + OpenRouter." By month 2, someone argues dynamic routing is "trivial to add" and ACE is "just a context layer." Timeline slips to 6 months. Momentum dies. A faster-moving competitor (another OpenCode fork, another claw-code derivative) fills the gap.

**Why it happens:**
- OSS projects die far more often from ambition than from technical difficulty.
- Each wedge (routing, ACE, verification-first, multi-agent) feels *close* because the harness is *almost* ready.
- Contributors pitch "small" additions that expand surface area nonlinearly.
- The benchmark target makes it tempting to add one more technique to squeeze another point.

**How to avoid:**
- The PROJECT.md Out of Scope list is a contract. Treat every proposal to move an item out of "Out of Scope" as a **new milestone**, not a v1 add.
- Require a written "what v1 feature does this replace" justification for any additive scope.
- Set a **date-based freeze**: by month 2, no new feature work, only hardening and eval.
- Track a "scope health" metric: closed-without-merging rate of scope-expanding issues. If it drops, the project is losing discipline.

**Warning signs:**
- Roadmap PRs that append rather than replace.
- "It's just a plugin" arguments.
- The eval score stops moving — team is building features instead of improving the harness.

**Phase to address:** All phases — scope freeze is a governance discipline. Phase 1 (governance) sets the contract.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode the benchmark config as default | Faster eval iteration | Interactive users hit TB-specific behaviors | Never (see Pitfall 3) |
| Skip NOTICE/attribution cleanup "for now" | Ship faster | License complaint, GitHub pull | Never (see Pitfall 2) |
| `.unwrap()` on tool-call JSON parsing | 20 LOC less code | Panic on provider variance | Never past PoC (see Pitfall 4) |
| Silent fallback to ICL parsing when tools fail | Fewer user-visible errors | Silent degradation, benchmark regressions invisible | Never (see Pitfall 10) |
| Fire `image_read` on every command | Simpler harness logic | Context window blown, cost explosion | Only when text parsing yields garbled output (see Pitfall 7) |
| Always run multi-perspective verification | Predictable quality | 3× token cost | Benchmark/release mode only (see Pitfall 6) |
| Run benchmarks nightly on main | Early regression detection | $3-10k/month unbudgeted | Only with a hard cost cap (see Pitfall 13) |
| Use `externalBin` for ForgeCode binary | Quick prototyping | macOS notarization breaks | Never for released builds (see Pitfall 8) |
| Auto-update via hand-hosted server | Full control | Extra infra, keys to rotate | Only after GitHub-Releases-based updater is proven (see Pitfall 8) |
| CLA instead of DCO | Future relicensing freedom | Contributor friction | Only if relicensing path is concrete (see Pitfall 11) |
| Copy a "clever" prompt from a blog post | Saves 10 min of prompt engineering | Provenance ambiguity (leak-tainted?) | Only when source is demonstrably clean-room (see Pitfall 12) |
| Single-threaded IPC emit for streaming tokens | Easy to code | Memory leak at 2h+ sessions | Never for release builds (see Pitfall 9) |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| OpenRouter tool calling | Trust that the OpenAI schema works uniformly | Maintain an allowlist; run per-model smoke tests (see Pitfall 10) |
| OpenRouter streaming | Assume `[DONE]` always arrives | Handle `finish_reason: 'stop'` with pending tool calls (Gemini-style); time-bound the stream |
| OpenRouter rate limits | Treat as OpenAI-equivalent | Check per-underlying-provider limits; OpenRouter's combined limit ≠ the provider's direct limit |
| OpenRouter cost attribution | Sum headers blindly | `X-OpenRouter-Usage` can differ from actual billing; reconcile via dashboard |
| Tauri auto-update | Ship update URL without version constraint | Pin the update signature key and version gate; rollback plan mandatory |
| Tauri IPC | `emit` for everything | Use channels for streaming; batch low-priority; separate high-frequency streams (see Pitfall 9) |
| Tauri sidecar (`externalBin`) | Bundle the harness as a sidecar | Merge into the main Rust binary to avoid notarization bug (see Pitfall 8) |
| macOS notarization | Run notarization only on release tags | Run on every merge to main so stalls surface early |
| Windows signing | Store certificate in repo secrets as a file | EV certs need hardware tokens or cloud HSM; automate via Azure Code Signing or DigiCert KeyLocker |
| ForgeCode upstream tracking | Fast-forward the fork every release | Pin a known-good commit; rebase on demand with a review pass (avoid upstream velocity risk) |
| Terminal-Bench Harbor harness | Run the harness and trust local scores | Match the official submission env exactly; reserve a held-out task subset (see Pitfall 1) |
| User config files edited while running | Reload-on-change is "convenient" | Detect change, prompt user to confirm, reload atomically — avoid mid-session schema drift |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Tauri renderer memory growth | UI laggy after hours of use | Channels over emit; batch events; periodic webview reload | 2-4 hour sessions (see Pitfall 9) |
| Unbounded context for long sessions | Token count monotonically increases | Implement summarization at 50% of window; evict images first (see Pitfall 7) |
| Re-indexing context engine on every turn | High CPU, slow first-turn | Incremental index with file-watch invalidation | Repos >10k files |
| Multi-perspective verification always on | Cost per task 3× baseline | Gate on confidence/complexity signal (see Pitfall 6) | Every session in production |
| Blocking the Tauri main thread from Rust | UI freezes during long agent turns | All agent work in `tokio::spawn` on a dedicated runtime; never `block_on` from a command handler | Any real-world use |
| Sync logging to disk inside the agent loop | Disk I/O stalls reasoning | Async append-only logger; batch flush | Sessions with >100 tool calls |
| Spawning a new subprocess per shell command | Process startup overhead dominates | Keep a long-lived PTY session per agent | Tasks with >20 shell commands |
| Polling the PTY at tight intervals | CPU-bound, battery drain on laptops | Adaptive polling; yield to OS read-ready events | Desktop use >30 min |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Agent runs arbitrary shell with user's credentials | Prompt injection → credential exfiltration | Subprocess sandbox (no network for untrusted commands, limited FS); explicit user consent for first-touch of a directory |
| Tauri `allowlist` overly permissive | Webview XSS → native API access | Minimal allowlist; CSP locked down; Tauri 2 capabilities configured per-window |
| Storing OpenRouter API key in plaintext config | Key exfiltration via repo commit, backup, sync | OS keychain (macOS Keychain, Windows Credential Manager, libsecret); never log the key |
| Auto-update without signature verification | MITM → malicious update | Tauri-plugin-updater signature required; public key pinned at build time |
| User-controlled prompts passed verbatim to tool arguments | Prompt injection → tool hijack | Separate user messages from tool arguments in the prompt template; quote/escape on render |
| Accepting tool calls for filesystem/shell without path allowlist | Agent writes to `/etc/`, `~/.ssh/` | Explicit workspace root; path normalization; deny traversal |
| Logging full tool-call payloads including secrets | Secrets in logs uploaded to issue tracker | Redact patterns (tokens, keys) before logging; user-visible redaction warning |
| Sidecar binary not integrity-checked at launch | Swap attack if app directory is writable | Verify sidecar hash at launch; fail-closed |
| Session files saved world-readable | Other users on the system read project secrets | 0600 on Unix; Tauri `app_data_dir` with proper ACLs on Windows |
| `image_read` on user screenshots without consent | Leaks sensitive screen content to the model provider | Visual indicator in UI when images are being captured; "pause vision" toggle |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Tauri UI hides the agent's raw output in favor of a "prettified" view | Users can't see what the agent actually ran; trust erodes | Pretty view by default, raw transcript one click away, always available |
| Cost shown only after the session ends | Surprise bill | Live token + $ counter per turn; projected session cost |
| No way to steer the agent mid-task from the desktop UI | User watches it go wrong, helpless | Interrupt button + "interject a message" lane (see Pitfall 5 for the PTY boundary) |
| Config changes require restart | Long debug loop for `~/.kay/config.toml` edits | Hot-reload with a visible reload banner; schema validation on save |
| OpenRouter model selector as a flat list of 300 models | Users pick the wrong model | Tiered list: "Recommended" (allowlisted), "Experimental" (limited support), "All" behind a warning |
| No recovery from a failed notarization/signed install | User re-installs, hits the same error | Clear error messages in the installer ("Apple couldn't notarize; visit [link]"); fall-back to unsigned dev build with warning |
| Sessions lost on crash | User re-does an hour of work | Incremental session persistence; resume prompt on next launch |
| Benchmark mode indistinguishable from interactive mode | Benchmark behaviors leak into interactive use (Pitfall 3) | Explicit mode toggle in the UI with distinct color/icon |

---

## "Looks Done But Isn't" Checklist

- [ ] **Apache-2.0 compliance:** NOTICE preserved verbatim — run `diff NOTICE upstream-ForgeCode/NOTICE`; file-level copyright headers intact; `LICENSE` unchanged.
- [ ] **Attribution visible:** README has a "Derived from ForgeCode" statement above the fold; `ATTRIBUTIONS.md` committed.
- [ ] **Code signing pipeline:** Signed+notarized build on PR merge to main, not just on release tag; certificate renewal date calendared.
- [ ] **Tauri memory hygiene:** 4-hour canary session documented with memory delta < threshold; subscribed to upstream Tauri IPC memory issues.
- [ ] **Tool-calling smoke test:** CI runs against every allowlisted OpenRouter model weekly; last-green date visible in README.
- [ ] **Benchmark submission parity:** Local harness config diffed against official Harbor submission settings; held-out task subset honored.
- [ ] **Cross-platform PTY:** Windows CI runs the full interactive PTY test suite on `windows-latest`; modern ConPTY flags verified in effect.
- [ ] **Budget caps:** `--max-usd` enforced in harness; no-cap full run requires explicit flag + confirmation.
- [ ] **Cost visibility:** Live token/USD meter in UI during session; per-session total exported with logs.
- [ ] **CLA/DCO enforcement:** Bot blocks unsigned PRs; drive-by contributor experience tested by a new GitHub account.
- [ ] **Clean-room provenance:** CONTRIBUTING.md explicitly prohibits reference to the Claude Code leak; clean-room log for prompts.
- [ ] **Auto-update signature:** Updater verifies signature with a pinned public key; rollback tested.
- [ ] **Secret handling:** API keys in OS keychain, not config files; redaction verified in logs; sample-bug-report template won't leak a key.
- [ ] **Marker scheme:** Cryptographically random sentinels; user-input-lane boundary explicit in UI; sentinel collision rejected at input validation.
- [ ] **Verification gating:** Multi-perspective verification fires only on confidence/complexity signal; default off for interactive mode.
- [ ] **`image_read` cap:** Per-turn and per-session image counts capped in tool wrapper, not just prompt.
- [ ] **Benchmark/interactive separation:** Distinct mode toggle; TB-specific phrases ("solution.sh", "run the tests") absent from interactive system prompt.
- [ ] **Upstream ForgeCode pinned:** `UPSTREAM_COMMIT.md` records base commit; manual rebase process documented.
- [ ] **Scope discipline:** v1 ships with exactly the Out of Scope list unchanged; any addition required a retrospective justification.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| NOTICE/attribution broken (Pitfall 2) | LOW | Patch on next release; public post-mortem; reach out to ForgeCode maintainers proactively. |
| TB 2.0 submission fails validation (Pitfall 1) | MEDIUM | Re-run on official Harbor env; audit for tuning artifacts; hold the public announcement until clean. |
| Tool-calling regression on one model (Pitfalls 4, 10) | LOW | Remove from allowlist immediately; release notes; re-test after upstream fix. |
| Tauri memory leak shipped (Pitfall 9) | MEDIUM | Hotfix release with event batching; recommend session length limit in docs until fixed. |
| Notarization stuck blocking release (Pitfall 8) | HIGH | Parallel path: ship unsigned "developer build" with loud warning; chase Apple support; consider sidecar removal. |
| Benchmark cost overrun (Pitfall 13) | LOW | OpenRouter usage dashboard; implement cost cap if not present; credit request to OpenRouter if egregious. |
| DMCA filed against the repo (Pitfall 12) | HIGH | GitHub counter-notice with clean-room evidence; legal review; backup repo ready on alternative host (Codeberg, SourceHut). |
| CLA deterring contributors (Pitfall 11) | MEDIUM | Survey failed-PR authors; consider DCO migration (requires commit history sign-off, not retroactive). |
| Scope creep derailed v1 (Pitfall 15) | HIGH | Public scope reset; extract deferred work into a v2 milestone; apologize to users. |
| Windows ConPTY regression (Pitfall 14) | MEDIUM | Pin to last-known-good `portable-pty` fork; add regression test; upstream a fix. |
| Marker collision in production (Pitfall 5) | LOW | Rotate sentinel format; add collision detection + abort; document for users. |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Benchmark overfitting | Phase 2 (harness), Phase 5 (submission) | Held-out task set scores within 2 pts of official; real-repo eval set passes |
| 2. License/NOTICE hygiene | Phase 1 (fork) | CI diff check; legal review pre-release |
| 3. ForgeCode over-specialization | Phase 2 + Phase 4 | "ForgeCode-isms" list empty; interactive mode soak test passes |
| 4. Non-spec tool-call JSON | Phase 2 (harness) | Fault injection tests; weekly smoke suite |
| 5. Marker race with user input | Phase 3 (KIRA) + Phase 4 (UI) | Collision test in CI; UI boundary documented |
| 6. Verification token cost | Phase 3 (KIRA) | Cost-per-task regression gate in CI |
| 7. `image_read` bloat | Phase 3 (KIRA) | Per-session image count capped in tool; context-usage dashboard |
| 8. Signing/notarization | Phase 1 (infra), Phase 5 (release) | Signed build on every main merge; installer smoke test on all three OSes |
| 9. Tauri memory leak | Phase 4 (UI) | 4-hour canary session, memory delta under threshold |
| 10. OpenRouter tool-call gaps | Phase 2 (provider) | Allowlist + weekly smoke test; "Compatibility unknown" UI warning for off-list models |
| 11. CLA friction | Phase 1 (governance) | Contributor retention metric; decision documented |
| 12. DMCA / leak proximity | Phase 1 (policy) + ongoing | Clean-room log; contributor attestation; legal counter-notice template ready |
| 13. Benchmark cost overrun | Phase 2 + Phase 5 | `--max-usd` enforced; cost dashboard reviewed at each phase transition |
| 14. Cross-platform PTY | Phase 2 (harness), Phase 5 (validation) | Windows CI green on full PTY suite |
| 15. Scope creep | All phases (governance) | Out-of-Scope list unchanged by scope-freeze date; closed-without-merge metric |

---

## Sources

**Terminal-Bench 2.0 structure and scoring:**
- [Terminal-Bench 2.0 Leaderboard (Morph LLM, 2026)](https://www.morphllm.com/terminal-bench-2) — 89 tasks, Docker + pytest scoring
- [Introducing Terminal-Bench 2.0 and Harbor (tbench.ai)](https://www.tbench.ai/news/announcement-2-0) — Harbor harness, 5-run submission average
- [Harbor framework GitHub](https://github.com/laude-institute/terminal-bench) — task structure, solution.sh, Daytona containers
- [Terminal-Bench arXiv paper](https://arxiv.org/html/2601.11868v1) — dataset construction, binary scoring

**OpenRouter tool-calling and streaming variance:**
- [OpenRouter Provider Variance — Introducing Exacto](https://openrouter.ai/announcements/provider-variance-introducing-exacto) — quality telemetry, per-provider tool-call accuracy gaps
- [OpenRouter Tool & Function Calling docs](https://openrouter.ai/docs/guides/features/tool-calling) — streaming coercion, missing-arguments handling
- [OpenRouter AI SDK — Tool Calls and Function Calling (DeepWiki)](https://deepwiki.com/OpenRouterTeam/ai-sdk-provider/4.3-tool-calls-and-function-calling) — Gemini `finish_reason: 'stop'` with pending tool calls
- [OpenRouter Streaming API reference](https://openrouter.ai/docs/api/reference/streaming)

**Tauri 2 signing, notarization, IPC memory leaks:**
- [Tauri macOS Code Signing docs](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri Windows Code Signing docs](https://v2.tauri.app/distribute/sign/windows/)
- [Tauri issue #11992 — sidecar/externalBin breaks notarization](https://github.com/tauri-apps/tauri/issues/11992)
- [Tauri discussion #8630 — notarization stuck for >1h](https://github.com/orgs/tauri-apps/discussions/8630)
- [Tauri issue #12724 — memory leak emitting events](https://github.com/tauri-apps/tauri/issues/12724)
- [Tauri issue #13133 — channel-event memory leak via `transformCallback`](https://github.com/tauri-apps/tauri/issues/13133)
- [Tauri issue #852 — renderer memory leak on Rust→JS events](https://github.com/tauri-apps/tauri/issues/852)
- ["Shipping a Production macOS App with Tauri 2.0" (DEV)](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3)

**Cross-platform PTY in Rust:**
- [portable-pty crate docs](https://docs.rs/portable-pty)
- [wezterm ConPTY implementation](https://github.com/wezterm/wezterm/blob/main/pty/src/win/conpty.rs) — modern ConPTY flags
- [winpty-rs crate](https://github.com/andfoy/winpty-rs) — ConPTY and WinPTY abstraction

**Claude Code leak / claw-code / DMCA:**
- [Claude Code Source Leak Guide (claudefa.st, 2026)](https://claudefa.st/blog/guide/mechanics/claude-code-source-leak) — 512k LOC, npm v2.1.88 source-map
- [Bean Kinney Korman — legal analysis of the leak and clean-room rewrites](https://www.beankinney.com/512000-lines-one-night-zero-permission-the-claude-code-leak-and-the-legal-crisis-of-ai-clean-rooms/)
- [Prism News — 8,100 DMCA takedowns](https://www.prismnews.com/news/anthropic-source-code-leak-triggers-mass-dmca-takedowns)
- ["Claw Code Killed Claude Code?" (Medium)](https://medium.com/data-science-in-your-pocket/claw-code-killed-claude-code-02aab80b0838) — claw-code clean-room provenance
- [36Kr — Claude Code rebranded, Anthropic blocking fails](https://eu.36kr.com/en/p/3747613304193796)

**CLA vs DCO and OSS governance:**
- [opensource.com — Why CLAs aren't good for open source](https://opensource.com/article/19/2/cla-problems)
- [Apache contributors need not sign a CLA (Petro)](https://apetro.ghost.io/apache-contributors-no-cla/)
- [ASF Contributor Agreements](https://www.apache.org/licenses/contributor-agreements.html)
- [Academy Software Foundation — DCO adoption](https://tac.aswf.io/process/contributing.html)

**PROJECT context:**
- `/Users/shafqat/Documents/Projects/opencode/vs-others/.planning/PROJECT.md` — Kay's active requirements, out-of-scope list, constraints, decisions.

---

*Pitfalls research for: Kay (open-source coding agent — Rust + Tauri + OpenRouter, forked from ForgeCode, targeting TB 2.0 >81.8%).*
*Researched: 2026-04-19.*
