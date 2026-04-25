#!/bin/sh
set -e
git add deny.toml .cargo/audit.toml \
  crates/kay-context/src/store.rs \
  crates/kay-tools/src/builtins/task_complete.rs \
  crates/kay-tools/src/events.rs \
  crates/kay-tools/src/runtime/context.rs

git commit -m "fix(ci): ignore RUSTSEC-2026-0104 + apply rustfmt to kay-tools/kay-context

RUSTSEC-2026-0104: rustls-webpki 0.101.7 reachable panic in CRL parsing
(published 2026-04-22). Transitive via rmcp/reqwest; Kay does not parse
CRLs directly. Patched in >=0.103.13; blocked on rmcp upstream upgrade.
Added to both deny.toml and .cargo/audit.toml (kept in sync per policy).

Also applies stable rustfmt to 4 kay-tools/kay-context source files that
cargo fmt --check flagged in the Lint CI job.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
