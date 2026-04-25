#!/bin/sh
set -e
# Check if tauri has a command macro
# The __cmd__ macros are in tauri-macros crate
# We might need the compression feature OR tauri-macros explicitly

# Let's try just using default features (no extra features on tauri)
python3 << 'PYEOF'
with open("crates/kay-tauri/Cargo.toml", "r") as f:
    content = f.read()

content = content.replace(
    'tauri             = { workspace = true, features = ["specta", "macros"] }',
    'tauri             = { workspace = true }'
)
content = content.replace(
    'tauri             = { workspace = true, features = ["test", "specta", "macros"] }',
    'tauri             = { workspace = true, features = ["test"] }'
)

with open("crates/kay-tauri/Cargo.toml", "w") as f:
    f.write(content)
print("Updated")
PYEOF
grep -n "^tauri" crates/kay-tauri/Cargo.toml
