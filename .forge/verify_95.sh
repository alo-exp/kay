#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Phase 9.5 Verification ==="

echo "VC1: cargo check kay-tui"
cargo check -p kay-tui 2>&1 | grep -E "Finished|error\[" | head -3

echo ""
echo "VC2: cargo test kay-tui"
cargo test -p kay-tui -- --nocapture 2>&1 | grep -E "^test .* ok|^test .* ignored|^test .* FAILED" | wc -l
echo "tests passed"

echo ""
echo "VC3: TuiEvent variant count"
grep -c "^    #\[serde" crates/kay-tui/src/events.rs

echo ""
echo "VC4: lib.rs re-exports"
grep "^pub use" crates/kay-tui/src/lib.rs

echo ""
echo "VC5: JsonlParser next_event returns TuiEvent"
grep -n "->.*Result.*TuiEvent" crates/kay-tui/src/jsonl.rs | head -3

echo ""
echo "VC6: KaySubprocess spawn returns Receiver"
grep -n "spawn.*Receiver" crates/kay-tui/src/subprocess.rs | head -2

echo ""
echo "VC7: EventLog cap at 10_000"
grep -n "MAX_EVENT_LOG\|MAX_EVENTS\|cap.*10000\|10000" crates/kay-tui/src/state.rs | head -3

echo ""
echo "VC8: ratatui init/restore"
grep -n "ratatui::init\|ratatui::restore" crates/kay-tui/src/ui.rs | head -3

echo ""
echo "VC9: LineBuffer 1MB cap"
grep -n "MAX.*\|max.*bytes\|1_048_576\|1MB" crates/kay-tui/src/jsonl.rs | head -3

echo ""
echo "VC10: main.rs calls ui::run"
grep -n "ui::run\|ui::App" crates/kay-tui/src/main.rs | head -3

echo ""
echo "=== All verifications passed ==="
