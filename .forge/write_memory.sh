#!/bin/sh
set -e
MEMDIR="$HOME/.claude/projects/-Users-shafqat-Documents-Projects-opencode-vs-others/memory"
mkdir -p "$MEMDIR"

cat > "$MEMDIR/feedback_forge_rust_patterns.md" << 'EOF'
---
name: Forge patterns for Rust workspace projects
description: Critical Forge config and usage patterns learned while shipping Kay Phase 8 — prevents timeouts, stalls, and silent failures
type: feedback
---

**Rule:** `tool_timeout_secs` (not `tool_timeout`) is the correct `.forge.toml` key. `tool_timeout` is silently ignored.

**Why:** Discovered when cargo commands kept timing out despite `tool_timeout = 720`. Set to `1200` for Rust workspaces.

**How to apply:** Always use `tool_timeout_secs = 1200`. Add `$schema = "https://forgecode.dev/schema.json"` for IDE validation.

---

**Rule:** Never run `cargo check/build/clippy/test` inline in `forge -p "..."`. Always delegate to a `.forge/script.sh` file.

**Why:** Cold Rust workspace compilation exceeds 5-minute default timeout. Forge retries inline commands without changing strategy.

**How to apply:** Write `.forge/check_crate.sh`, `.forge/commit_<task>.sh` etc. Run foreground: `forge -p "BRAIN ROLE: Run: sh .forge/script.sh 2>&1"`.

---

**Rule:** Never embed multi-line git commit messages in `forge -p "..."` prompts. Use `.forge/commit_<task>.sh` scripts.

**Why:** Newline + single-quote combos inside double-quoted forge prompts cause the background task to hang indefinitely (1-line empty output, no error).

**How to apply:** Write commit+push to a `.forge/commit_*.sh` script, run foreground.

---

**Rule:** Always run forge foreground (not `run_in_background: true`) for multi-line/complex operations.

**Why:** Background Forge tasks stall silently on complex prompts — file stays at 1 line forever.

**How to apply:** `Bash({ command: "forge -p '...' 2>&1 | tail -N" })` always foreground.
EOF

# Update MEMORY.md index
MEMORY_FILE="$MEMDIR/MEMORY.md"
if ! grep -q "feedback_forge_rust_patterns" "$MEMORY_FILE" 2>/dev/null; then
  echo "- [Forge Rust patterns](feedback_forge_rust_patterns.md) — tool_timeout_secs key, .forge/ script pattern, no background tasks for multi-line ops" >> "$MEMORY_FILE"
fi

echo "MEMORY_WRITTEN_OK"
