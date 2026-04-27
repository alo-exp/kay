import re

with open("Cargo.toml", "r") as f:
    content = f.read()

# Fix tauri-specta line (was duplicated)
content = content.replace(
    'tauri-specta      = { version = "=2.0.0-rc.22", features = ["derive"] }", features = ["derive"] }',
    'tauri-specta      = { version = "=2.0.0-rc.22", features = ["derive"] }'
)
content = content.replace(
    'specta            = { version = "=2.0.0-rc.22" }" }',
    'specta            = { version = "=2.0.0-rc.22" }'
)

with open("Cargo.toml", "w") as f:
    f.write(content)

print("Fixed")
