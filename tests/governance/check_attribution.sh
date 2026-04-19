#!/usr/bin/env bash
# tests/governance/check_attribution.sh
# Phase 1 governance invariant verifier.
# Run locally or from `/gsd:verify-work`; covers GOV-01, GOV-02, GOV-04, GOV-06, GOV-07.
# Exits 0 on success, non-zero on any failure (prints which invariant failed).

set -euo pipefail

# --- color (only when stdout is a tty) ---
if [ -t 1 ]; then
  C_GREEN=$'\033[32m'
  C_RED=$'\033[31m'
  C_RESET=$'\033[0m'
else
  C_GREEN=""
  C_RED=""
  C_RESET=""
fi

usage() {
  cat <<'EOF'
Usage: tests/governance/check_attribution.sh [--help]

Run Phase 1 governance invariant assertions. Exits 0 if every check
passes, non-zero (= number of failures) otherwise.

Checks performed:
  GOV-01 / GOV-02 -- Apache-2.0 LICENSE + NOTICE attribution
    - LICENSE exists and contains "Apache License"
    - NOTICE exists and cites ForgeCode, Tailcall, Apache-2.0
    - NOTICE is brief (<= 20 lines)
    - NOTICE has no "<SHA>" placeholder
  GOV-01       -- README.md Acknowledgments section
    - README.md has "## Acknowledgments"
    - README mentions ForgeCode and Terminus-KIRA
  GOV-01       -- ATTRIBUTIONS.md
    - ATTRIBUTIONS.md exists
    - No "<UPSTREAM_COMMIT>" placeholder
    - Cites rust-toolchain 1.95 divergence
  GOV-04/GOV-07 -- CONTRIBUTING.md (DCO + clean-room)
    - Cites "Developer Certificate of Origin" and developercertificate.org
    - Shows "git commit -s"
    - Clean-room: v2.1.88, 2026-03-31, @anthropic-ai/claude-code
    - References SECURITY.md
  GOV-06       -- SECURITY.md
    - Mentions Security Advisory, security@kay.dev, docs/signing-keys, git tag -v
  Crate-level NOTICE for derived source
    - crates/kay-core/NOTICE exists and cites Tailcall
  Fork SHA record
    - .forgecode-upstream-sha exists and is 40-char hex
  forgecode-parity-baseline tag
    - Tag exists and is annotated (unsigned per D-OP-04 amendment)

Must be run from the Kay repository root.
EOF
}

# Parse flags
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

failed=0

check() {
  local name="$1"; shift
  if "$@" >/dev/null 2>&1; then
    printf '  %sPASS%s: %s\n' "$C_GREEN" "$C_RESET" "$name"
  else
    printf '  %sFAIL%s: %s\n' "$C_RED" "$C_RESET" "$name"
    failed=$((failed + 1))
  fi
}

# Inverse variant: passes when the command FAILS (e.g. grep finds nothing).
# Needed because bash `!` operator can't be passed through "$@".
check_absent() {
  local name="$1"; shift
  if "$@" >/dev/null 2>&1; then
    printf '  %sFAIL%s: %s\n' "$C_RED" "$C_RESET" "$name"
    failed=$((failed + 1))
  else
    printf '  %sPASS%s: %s\n' "$C_GREEN" "$C_RESET" "$name"
  fi
}

echo "== GOV-01 / GOV-02: Apache-2.0 LICENSE + NOTICE attribution =="
check "LICENSE exists"                         test -f LICENSE
check "LICENSE contains 'Apache License'"      grep -q 'Apache License' LICENSE
check "NOTICE exists"                          test -f NOTICE
check "NOTICE cites ForgeCode"                 grep -q 'ForgeCode' NOTICE
check "NOTICE cites Tailcall copyright"        grep -q 'Tailcall' NOTICE
check "NOTICE cites Apache-2.0"                grep -q 'Apache License, Version 2.0' NOTICE
check "NOTICE brief (<= 20 lines)"             test "$(wc -l < NOTICE)" -le 20
check_absent "NOTICE has no '<SHA>' placeholder" grep -q '<SHA>' NOTICE

echo "== GOV-01: README.md Acknowledgments =="
check "README.md exists"                       test -f README.md
check "README has ## Acknowledgments"          grep -q '## Acknowledgments' README.md
check "README mentions ForgeCode"              grep -q 'ForgeCode' README.md
check "README mentions Terminus-KIRA"          grep -q 'Terminus-KIRA' README.md

echo "== GOV-01: ATTRIBUTIONS.md =="
check "ATTRIBUTIONS.md exists"                 test -f ATTRIBUTIONS.md
check_absent "ATTRIBUTIONS has no '<UPSTREAM_COMMIT>' placeholder" grep -q '<UPSTREAM_COMMIT>' ATTRIBUTIONS.md
check "ATTRIBUTIONS cites rust-toolchain 1.95 divergence" grep -q '1.95' ATTRIBUTIONS.md

echo "== GOV-04 / GOV-07: CONTRIBUTING.md (DCO + clean-room) =="
check "CONTRIBUTING.md exists"                 test -f CONTRIBUTING.md
check "CONTRIBUTING cites DCO"                 grep -q 'Developer Certificate of Origin' CONTRIBUTING.md
check "CONTRIBUTING links developercertificate.org" grep -q 'developercertificate.org' CONTRIBUTING.md
check "CONTRIBUTING shows 'git commit -s'"     grep -q 'git commit -s' CONTRIBUTING.md
check "CONTRIBUTING clean-room cites Claude Code leak version" grep -q 'v2.1.88' CONTRIBUTING.md
check "CONTRIBUTING clean-room cites leak date" grep -q '2026-03-31' CONTRIBUTING.md
check "CONTRIBUTING cites '@anthropic-ai/claude-code'" grep -q '@anthropic-ai/claude-code' CONTRIBUTING.md
check "CONTRIBUTING references SECURITY.md"    grep -q 'SECURITY.md' CONTRIBUTING.md

echo "== GOV-06: SECURITY.md =="
check "SECURITY.md exists"                     test -f SECURITY.md
check "SECURITY mentions 'Security Advisory'"  grep -q 'Security Advisory' SECURITY.md
check "SECURITY has disclosure fallback email" grep -q 'security@kay.dev' SECURITY.md
check "SECURITY references signing-keys dir"   grep -q 'docs/signing-keys' SECURITY.md
check "SECURITY cites git tag -v"              grep -q 'git tag -v' SECURITY.md

echo "== crate-level NOTICE for derived source =="
check "crates/kay-core/NOTICE exists"          test -f crates/kay-core/NOTICE
check "kay-core/NOTICE cites Tailcall"         grep -q 'Tailcall' crates/kay-core/NOTICE

echo "== fork SHA record =="
check ".forgecode-upstream-sha exists"         test -f .forgecode-upstream-sha
check ".forgecode-upstream-sha is 40-char hex" grep -qE '^[0-9a-f]{40}' .forgecode-upstream-sha

echo "== forgecode-parity-baseline tag (unsigned per D-OP-04 amendment) =="
if git rev-parse --git-dir >/dev/null 2>&1; then
  check "tag forgecode-parity-baseline exists" git rev-parse forgecode-parity-baseline
  if git rev-parse forgecode-parity-baseline >/dev/null 2>&1; then
    check "tag is annotated (not lightweight)" test "$(git cat-file -t forgecode-parity-baseline 2>/dev/null)" = "tag"
  fi
fi

echo
if [ "$failed" -eq 0 ]; then
  printf '%sALL GOVERNANCE INVARIANTS PASS%s\n' "$C_GREEN" "$C_RESET"
  exit 0
else
  printf '%sFAILED: %d invariant(s) -- fix and re-run%s\n' "$C_RED" "$failed" "$C_RESET"
  exit 1
fi
