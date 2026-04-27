#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-config/
git add crates/kay-cli/src/run.rs
git add crates/kay-cli/src/interactive.rs
git add crates/kay-cli/Cargo.toml
git add Cargo.toml

git commit -m "feat(kay-config): add TOML configuration system

Phase 12: Enable configuration via config TOML files (like Forge).

New crate: kay-config with:
- KayConfig struct for layered config (defaults + user file + env vars)
- Reads from: embedded kay.toml → ~/.kay/kay.toml → KAY_* env vars
- API key resolution from env vars or config
- Provider settings (endpoint, api_key_var)
- API settings (max_tokens, temperature, timeout)
- Session settings (max_turns, persist)

Updated kay-cli to use KayConfig:
- run.rs: Load config, pass to live_provider
- interactive.rs: Load config for run_live_turn
- Model: CLI flag > config > embedded default
- API settings: config provides max_tokens, temperature

User config location: ~/.kay/kay.toml
Or set KAY_CONFIG env var for custom path

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
