## Summary

Five bugs in `hooks/forge-delegation-enforcer.sh` were discovered in a live session. They range from classification false-positives that incorrectly deny valid commands, to documented bypasses that silently fail, to a missing CLI entry, to two security holes that allow mutating commands and file writes to completely bypass forge-delegation. All five are in v1.2.2.

---

## Bug 1 — `has_write_redirect` over-matches `>` in non-redirect contexts

**Severity:** High — causes incorrect denials of valid commands  
**File/lines:** `hooks/forge-delegation-enforcer.sh` lines 281–297

### Root cause

```bash
has_write_redirect() {
  [[ "$cmd" == *">"* ]] || return 1   # naive match on entire raw string
  local pruned="$cmd"
  pruned="${pruned//>\/dev\/null/}"
  # only /dev/null patterns are pruned — everything else treated as a write redirect
  [[ "$pruned" == *">"* ]]
}
```

The naive `*">"*` string match fires on **any** `>` character in the command string, including:
- Heredoc bodies: `python3 << 'EOF'\nlet x: Vec<String> = ...\nEOF` — the `>` in Rust/TypeScript generics inside the heredoc body triggers this
- Quoted argument strings: `echo "Result<T, E>"` — the `>` in the string triggers this
- Standard fd redirects `>&1`, `>&2`, `2>&1` — only `2>&1` is pruned; `>&1` is not

Because `is_read_only` also calls `has_write_redirect` first (lines 255–258), **even `cd`, `cat`, `echo` commands are denied** if their arguments contain `>` in non-redirect contexts.

### Observed failure

```bash
cd /path && python3 << 'PYEOF'
# body contained: Result<String, String>
PYEOF
# → denied: Sidekick /forge mode: mutating command denied
```

### Fix

Replace the naive string match with a tokenizer-aware redirect detector. At minimum:
1. Strip heredoc bodies (content between `<<EOF` and `EOF`) before checking
2. Strip content inside single-quoted and double-quoted strings
3. Also strip `>&N` and `N>&M` patterns that do not write to files
4. Consider using `bash -n` (no-execute parse) or `python3 shlex` to identify actual redirect tokens

---

## Bug 2 — `FORGE_LEVEL_3=1` command-prefix bypass silently fails

**Severity:** High — the only documented Level-3 bypass is completely non-functional  
**File/lines:** `hooks/forge-delegation-enforcer.sh` lines 88, 216–224, 402–407

### Root cause

The bypass check reads the hook's **own process environment**:
```bash
if [[ "${FORGE_LEVEL_3:-}" == "1" ]]; then
    return 0  # Level-3 passthrough
fi
```

But `strip_env_prefix` only strips `FORGE_LEVEL_3=1` from the command **string** for classification purposes — it never exports the variable to the hook's own shell environment:
```bash
strip_env_prefix() {
  # strips KEY=val prefixes for classification ONLY — no export
  while [[ "$cmd" =~ ^[[:space:]]*[A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*[[:space:]]+ ]]; do
    cmd="${cmd#"${BASH_REMATCH[0]}"}"
  done
}
```

When Claude Code delivers `FORGE_LEVEL_3=1 some_command` as the Bash input, the PreToolUse hook runs in its own subprocess. `FORGE_LEVEL_3` is never in that subprocess's environment — it only existed as text in the `command` JSON field.

### Compounding documentation bug

The deny reason (line 88) actively misleads:
```
"...To temporarily bypass for Level 3 takeover, set FORGE_LEVEL_3=1 in the Bash environment."
```
Users following this instruction write `FORGE_LEVEL_3=1 <cmd>` and the bypass silently fails every time.

### Fix (choose one)

**Option A (minimal):** In `strip_env_prefix` (or in `decide_bash` before the classification calls), parse the stripped env-var prefixes and export them into the current shell:
```bash
# After strip_env_prefix, export what was stripped:
while [[ "$orig_cmd" =~ ^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)=([^[:space:]]*)[[:space:]]+ ]]; do
  export "${BASH_REMATCH[1]}"="${BASH_REMATCH[2]}"
  orig_cmd="${orig_cmd#"${BASH_REMATCH[0]}"}"
done
```

**Option B (safer):** Remove the command-prefix bypass entirely. Update the deny reason to say the Level-3 bypass requires running `/forge:deactivate` first, not a command prefix.

---

## Bug 3 — `gh` GitHub CLI is unclassified → denied

**Severity:** Medium — blocks all Brain-role GitHub operations  
**File/lines:** `hooks/forge-delegation-enforcer.sh` lines 299–323, 410

### Root cause

`gh` appears in neither `is_read_only` nor `is_mutating`. The enforcer's unclassified deny fires:
```bash
# 4. Unclassified → conservative deny.
emit_decision "deny" "Sidekick /forge mode: command could not be classified..."
```

### Observed failures

```bash
cd /path && ITEM_ID=$(gh project item-add 4 ...) && gh project item-edit ...
# → "Sidekick /forge mode: command could not be classified."

# Note: the cd-prefix workaround (Bug 4 below) was needed to even get
# gh issue create to work; the above compound assignment was still blocked
```

### Affected operations

All `gh` invocations are blocked: `gh issue create/list/view`, `gh pr create/list/merge`, `gh project item-add/edit`, `gh release create`, `gh label list`, etc.

### Fix

Add `gh` to the `is_mutating` list (line 309). It makes external API mutations. Optionally, add specific read-only sub-commands as two-word entries in `is_read_only` to match the `git status`/`git log` pattern:
```bash
# is_read_only two-word entries:
"gh issue list"|"gh pr list"|"gh pr view"|"gh issue view"|"gh label list"|"gh release list") return 0 ;;

# is_mutating single-word entry:
gh) return 0 ;;
```

---

## Bug 4 — `cd /path && mutating_cmd` chain bypasses classification (security)

**Severity:** High — security hole; any mutating command can bypass forge-delegation  
**File/lines:** `hooks/forge-delegation-enforcer.sh` lines 228–234, 265–277

### Root cause

`first_token` extracts only the first 1–2 tokens from the full command. `is_read_only` sees `cd` and returns true for the entire `&&`-chained string:

```bash
first_token() {
  printf '%s' "$stripped" | awk '{ if (NF>=2) { print $1" "$2 } else { print $1 } }'
  # returns "cd" for "cd /path && rm -rf /important && forge -p ..."
}

is_read_only() {
  # ...
  case "${first%% *}" in
    cd|ls|cat|...)  return 0 ;;   # cd → entire chain allowed
  esac
}
```

### Attack / bypass surface

Any command can bypass forge-delegation by prefixing with `cd /any/path &&`:
```bash
# All of these are allowed through:
cd /tmp && rm -rf /important/directory
cd . && curl -X DELETE https://api.example.com/resource
cd /src && git push --force origin main
cd /src && pip install malicious-package
```

### Discovery context

This was exploited in this session as a workaround for Bug 3: `cd /path && gh issue create ...` succeeded because `cd` is read-only and the hook never examined the `gh` segment.

### Fix

In `is_read_only` and `is_mutating`, detect command-chain connectors and validate ALL segments:
```bash
# Before classification, check if this is a chain
if [[ "$cmd" =~ [^|][|][^|] ]] || [[ "$cmd" =~ \&\& ]] || [[ "$cmd" =~ \; ]]; then
  # Split on &&, ||, ; and check each segment independently
  # A chain is read-only only if ALL segments are read-only
  # A chain is mutating if ANY segment is mutating
fi
```

---

## Bug 5 — MCP filesystem tools bypass enforcer entirely (security)

**Severity:** High — completely undermines forge-delegation for file writes  
**File/lines:** `hooks/forge-delegation-enforcer.sh` lines 446–451

### Root cause

The `case "$tool_name"` dispatch only handles three Claude Code native tool names. The wildcard passes everything else silently:

```bash
case "$tool_name" in
    Write|Edit)     decide_write_edit "$tool_input" ;;
    NotebookEdit)   decide_notebook_edit "$tool_input" ;;
    Bash)           decide_bash "$tool_input" ;;
    *)              exit 0 ;;    # ← all MCP tools pass through silently
esac
```

### Affected MCP tools

MCP tools that write files but are never intercepted:
- `mcp__filesystem__write_file` — creates/overwrites files
- `mcp__filesystem__edit_file` — patches file content
- `mcp__filesystem__move_file` — moves/renames files
- `mcp__filesystem__create_directory` — creates directories

### Observed exploitation

In this session, `mcp__filesystem__write_file` was used to write `.forge/` files and `.planning/` files directly to project repositories while forge-delegation was fully active. The hook never fired.

### Fix

Add explicit deny cases for MCP filesystem write tools:
```bash
case "$tool_name" in
    Write|Edit)     decide_write_edit "$tool_input" ;;
    NotebookEdit)   decide_notebook_edit "$tool_input" ;;
    Bash)           decide_bash "$tool_input" ;;
    # MCP filesystem write tools — same policy as Write/Edit
    mcp__filesystem__write_file|\
    mcp__filesystem__edit_file|\
    mcp__filesystem__move_file)
        deny_direct_edit ;;
    # mcp__filesystem__create_directory: deny for consistency
    mcp__filesystem__create_directory)
        deny_direct_edit ;;
    *)              exit 0 ;;
esac
```

Alternatively, use a pattern match if the Claude Code hook dispatch supports it:
```bash
mcp__filesystem__write_file|mcp__filesystem__edit_file|mcp__filesystem__move_file|mcp__filesystem__create_directory)
    deny_direct_edit ;;
```

---

## Prioritization

| # | Bug | Severity | Type |
|---|-----|----------|------|
| 4 | `cd &&` chain bypass | 🔴 High | Security hole |
| 5 | MCP filesystem bypass | 🔴 High | Security hole |
| 1 | `has_write_redirect` false-positives | 🔴 High | Incorrect denial |
| 2 | `FORGE_LEVEL_3` bypass silently fails | 🔴 High | Broken feature |
| 3 | `gh` unclassified → denied | 🟡 Medium | Missing allowlist entry |

Bugs 4 and 5 are security holes that should be fixed first — they allow complete bypass of the delegation system. Bug 1 causes widespread false positives that make forge-delegation frustrating to use. Bug 2 makes the Level-3 escape hatch non-functional. Bug 3 blocks common Brain-role operations.

## Affected version

v1.2.2 (`~/.claude/plugins/cache/alo-exp/sidekick/1.2.2/hooks/forge-delegation-enforcer.sh`)

## Labels

`bug`, `forge-delegation`, `security`
