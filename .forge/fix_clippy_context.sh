#!/bin/sh
set -e

FILE="crates/kay-tools/src/runtime/context.rs"

# Insert #[allow(clippy::too_many_arguments)] before the line containing "    pub fn new("
# at line 110 (context.rs ToolCallContext::new)
sed -i '' 's/    \/\/\/ `sage_query`'"'"'s inner context passes a fresh empty string (each/    \/\/\/ `sage_query`'"'"'s inner context passes a fresh empty string (each/' "$FILE"

# Use python for reliable multi-line insert
python3 - "$FILE" <<'PYEOF'
import sys
path = sys.argv[1]
with open(path, 'r') as f:
    lines = f.readlines()

allow_line = '    #[allow(clippy::too_many_arguments)]\n'
new_lines = []
for i, line in enumerate(lines):
    if line.strip() == 'pub fn new(' and allow_line not in lines[i-1:i]:
        new_lines.append(allow_line)
    new_lines.append(line)

with open(path, 'w') as f:
    f.writelines(new_lines)
print("Inserted allow attribute")
PYEOF

cargo fmt -p kay-tools

git add crates/kay-tools/src/runtime/context.rs
git commit -m "fix(clippy): allow too_many_arguments on ToolCallContext::new

Phase 8 W-4 added the 8th parameter task_context; suppress the
clippy::too_many_arguments lint rather than restructure the constructor.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
