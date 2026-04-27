#!/bin/sh
set -e

# Update workspace tauri-specta to include derive feature
sed -i '' 's/tauri-specta.*=2.0.0-rc.24.*/tauri-specta      = { version = "=2.0.0-rc.24", features = ["derive"] }/' Cargo.toml

# Update kay-tauri Cargo.toml: tauri dependency needs ["codegen"] for generate_handler!
# The specta feature needs to be in workspace tauri-specta AND in the tauri dependency

# Add specta feature to kay-tauri's tauri dependency
python3 << 'PYEOF'
with open("crates/kay-tauri/Cargo.toml", "r") as f:
    content = f.read()

content = content.replace(
    'tauri             = { workspace = true }',
    'tauri             = { workspace = true, features = ["macros"] }'
)

content = content.replace(
    'tauri             = { workspace = true, features = ["test"] }',
    'tauri             = { workspace = true, features = ["test", "macros"] }'
)

with open("crates/kay-tauri/Cargo.toml", "w") as f:
    f.write(content)
print("Updated kay-tauri Cargo.toml")
PYEOF

grep -n "^tauri" crates/kay-tauri/Cargo.toml
