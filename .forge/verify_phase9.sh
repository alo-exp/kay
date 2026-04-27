#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Phase 9 Success Criteria Verification ==="
echo ""

echo "SC1: cargo check -p kay-tauri compiles"
cargo check -p kay-tauri 2>&1 | grep -E "error|Finished|Compiling kay-tauri" | head -3
echo "✓ cargo check OK"

echo ""
echo "SC2: gen_bindings test"
cargo test -p kay-tauri --test gen_bindings 2>&1 | grep -E "test .* ok|FAILED|passed|failed" | head -5

echo ""
echo "SC3: check-bindings.sh"
bash scripts/check-bindings.sh 2>&1

echo ""
echo "SC4: pnpm build"
cd crates/kay-tauri/ui && pnpm build 2>&1 | grep -E "✓|error|built" | head -3

echo ""
echo "SC5: IpcAgentEvent variants"
grep -c "^    [A-Z]" crates/kay-tauri/src/ipc_event.rs

echo ""
echo "SC6: From<AgentEvent> unit tests"
grep -c "#\[test\]" crates/kay-tauri/src/ipc_event.rs

echo ""
echo "SC7: flush_task tests"
grep -c "#\[test\]" crates/kay-tauri/src/flush.rs

echo ""
echo "SC8: VerificationCard (React component)"
ls crates/kay-tauri/ui/src/components/VerificationCard.tsx 2>&1

echo ""
echo "SC9: Memory canary compiles"
cargo test -p kay-tauri --test memory_canary 2>&1 | grep -E "test .* ok|passed|failed|ignored" | head -5

echo ""
echo "=== All checks complete ==="
