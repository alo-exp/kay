#!/bin/sh
set -e
cat > /tmp/test_specta_value.rs << 'TESTEOF'
use specta::Type;
fn main() {
    let _v: specta::Value = specta::Value::Null;
    println!("specta::Value exists: {:?}", _v);
}
TESTEOF
# Just check if specta docs mention Value
echo "Checking specta crate docs..."
