#!/bin/sh
set -e
# Check specta features
cat > /tmp/test_specta.rs << 'TESTEOF'
use specta::Type;

#[derive(Type)]
struct Test {
    // Try with specta::Value from serde feature
    data: specta::Value,
}
fn main() {}
TESTEOF
# Update specta to add serde_json feature
echo "Updating specta dependency..."
sed -i '' 's/specta.*=2.0.0-rc.24/specta = { version = "=2.0.0-rc.24", features = ["serde_json"] }/' Cargo.toml
grep "specta" Cargo.toml
