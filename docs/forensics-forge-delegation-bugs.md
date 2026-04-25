# Forensic Report — Forge Delegation Enforcer Bugs

**Generated:** 2026-04-24T01:56:01Z
**Source file:** `~/.claude/plugins/cache/alo-exp/sidekick/1.2.2/hooks/forge-delegation-enforcer.sh`

## Bug 1 — `has_write_redirect` false-positives on non-redirect `>` (lines 281–297)

`[[ "$cmd" == *">"* ]]` fires on any `>` in the raw command string — heredoc bodies, angle-bracket generics in Rust/TypeScript code (`Vec<String>`, `Result<T, E>`), and quoted strings containing `>`. Only `/dev/null` patterns are pruned; all other occurrences cause the command to be classified as mutating even when no file is actually being written.

Observed in session: `python3 << 'PYEOF'` with Rust generics in the heredoc body was denied.

## Bug 2 — `FORGE_LEVEL_3=1` command-prefix bypass silently fails (lines 216–224, 402–407)

`strip_env_prefix` strips `FORGE_LEVEL_3=1` from the command string for classification purposes only — it never exports the variable to the hook's own subprocess environment. `${FORGE_LEVEL_3:-}` in the hook is always empty when the user writes `FORGE_LEVEL_3=1 <cmd>`. The bypass is completely non-functional via the documented command-prefix mechanism. The deny reason on line 88 falsely states this works.

## Bug 3 — `gh` CLI unclassified → denied (line 410)

`gh` is absent from both `is_read_only` and `is_mutating`, so all `gh` invocations fall to the conservative unclassified-deny path. Blocks all Brain-role operations: `gh issue create`, `gh pr create`, `gh project item-add`, `gh project item-edit`, etc.

## Bug 4 — `cd /path && mutating_cmd` chain bypasses classification (lines 228–234, 265–277)

`first_token` returns only the first 1–2 tokens of the full command. `is_read_only` sees `cd` → returns true → the ENTIRE `&&`-chained command passes through without examining subsequent segments. Any mutating command can bypass forge-delegation by prefixing with `cd /some/path &&`.

## Bug 5 — MCP filesystem tools bypass enforcer entirely (lines 446–451)

The `case "$tool_name"` dispatch only intercepts `Write`, `Edit`, `NotebookEdit`, `Bash`. The wildcard `*) exit 0` passes all other tool names silently. MCP tools (`mcp__filesystem__write_file`, `mcp__filesystem__edit_file`, `mcp__filesystem__create_directory`, `mcp__filesystem__move_file`) are never intercepted, allowing direct file writes that completely bypass forge-delegation.
