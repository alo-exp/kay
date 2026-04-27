#!/bin/sh
set -e
# Find specta-typescript entry in Cargo.lock and show its dependencies
python3 << 'PYEOF'
with open("Cargo.lock", "r") as f:
    content = f.read()
    
# Find all [[package]] blocks for specta-typescript
import re
entries = re.findall(r'\[\[package\]\]\nname = "specta-typescript"\nversion = "([^\"]+)"(.*?)(?=\n\[|$)', content, re.DOTALL)
for i, (version, deps) in enumerate(entries):
    print(f"=== Entry {i+1}: specta-typescript@{version} ===")
    print(deps.strip()[:300])
    print()
PYEOF
