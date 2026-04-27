#!/bin/sh
set -e
# The __cmd__start_session macro is defined by tauri's command attribute
# Check if "macros" feature was the right one

# Try adding both specta AND macros
python3 << 'PYEOF'
with open("crates/kay-tauri/Cargo.toml", "r") as f:
    content = f.read()

content = content.replace(
    'tauri             = { workspace = true, features = ["specta"] }',
    'tauri             = { workspace = true, features = ["specta", "macros"] }'
)
content = content.replace(
    'tauri             = { workspace = true, features = ["test", "specta"] }',
    'tauri             = { workspace = true, features = ["test", "specta", "macros"] }'
)

with open("crates/kay-tauri/Cargo.toml", "w") as f:
    f.write(content)
print("Updated")
PYEOF
grep -n "^tauri" crates/kay-tauri/Cargo.toml
