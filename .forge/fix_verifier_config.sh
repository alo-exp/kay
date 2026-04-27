#!/bin/sh
# Patch all RunTurnArgs struct literals to add verifier_config field.
# Uses full path kay_verifier::VerifierConfig — no import needed.
set -e
cd "$(dirname "$0")/.."

DISABLED_CONFIG='verifier_config: kay_verifier::VerifierConfig { mode: kay_verifier::VerifierMode::Disabled, max_retries: 0, cost_ceiling_usd: 0.0, model: String::new() },'

# Files with 8-space field indentation (4-space closing }))
FILES_8="
crates/kay-core/tests/loop.rs
crates/kay-core/tests/loop_dispatcher_integration.rs
crates/kay-core/tests/loop_sage_query_integration.rs
crates/kay-core/tests/loop_pause_tool_call_buffered.rs
"

for f in $FILES_8; do
  echo "Patching $f (8-space)"
  perl -i -0pe "s/        initial_prompt: String::new\(\),\n    \}\)\)/        initial_prompt: String::new(),\n        ${DISABLED_CONFIG}\n    }))/g" "$f"
done

# loop_property.rs has 12-space fields inside rt.block_on closure
echo "Patching loop_property.rs (12-space)"
perl -i -0pe "s/            initial_prompt: String::new\(\),\n        \}\)\)/            initial_prompt: String::new(),\n            ${DISABLED_CONFIG}\n        }))/g" \
  crates/kay-core/tests/loop_property.rs

echo "=== cargo check ==="
cargo check -p kay-core --tests 2>&1

echo "=== Result: $? ==="
