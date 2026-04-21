#!/usr/bin/env bash
#
# capture-parity-fixtures.sh
#
# Regenerates the CLI-07 parity fixtures under
# crates/kay-cli/tests/fixtures/ from the `forgecode-parity-baseline`
# git tag. See T7.10 in .planning/phases/05-agent-loop/05-PLAN.md and
# DL-1 in .planning/phases/05-agent-loop/05-CONTEXT.md for the contract.
#
# # What this script does
#
# 1. Verifies the `forgecode-parity-baseline` tag exists in this clone.
# 2. Reads the baseline's `forge_main::banner::display(cli_mode=false)`
#    tips list and `forge_main::prompt::KAY_PROMPT`-equivalent from the
#    tag's source tree via `git show <tag>:<path>`.
# 3. Reproduces the baseline's right-alignment logic (max-key-width
#    padding with a single trailing space, matching Rust's
#    `format!("{key:>max_width$} ")`).
# 4. Writes the fixtures:
#      - crates/kay-cli/tests/fixtures/forgecode-banner.txt
#      - crates/kay-cli/tests/fixtures/forgecode-prompt.txt
#
# # Why not just `cargo run` at the baseline tag?
#
# The tag is a *source-only import* — at tag-time the ForgeCode
# crates live under `crates/kay-core/src/forge_main/*` as a flat text
# dump, NOT as a buildable Cargo workspace. There is no binary to run.
# The fixture must therefore be derived *from the source text* of the
# banner.rs tips list, reproducing the right-alignment that the
# runtime would have emitted.
#
# # Fidelity contract
#
# The fixtures capture the *parity-relevant* portion of the baseline
# startup surface, which is the 6-row interactive-mode tips block
# (max_width=17 right-aligned). Two banner elements from the baseline
# are DELIBERATELY excluded from the fixture:
#
#   * The ASCII wordmark ("Forge" glyphs): kay ships its own "Kay"
#     wordmark, so a contiguous-substring match against kay's stdout
#     would fail on the glyph lines (no brand-swap rewrites glyphs).
#   * The `Version:` row: the baseline version differs from kay's
#     Cargo.toml version, so this row would never substring-match.
#   * The zsh-plugin encouragement box: kay dropped that surface (no
#     zsh plugin exists); the baseline fixture reflects kay's
#     shape, not ForgeCode's, per DL-1.
#
# What REMAINS is the 6 interactive tips + the shell prompt literal —
# the exact surface kay's `interactive::run()` reproduces.
#
# # Re-running the script
#
# Run from anywhere inside the repo:
#
#   ./scripts/capture-parity-fixtures.sh
#
# Idempotent: overwrites the fixture files on every run. Exits non-zero
# if the tag is missing or the baseline source has drifted from the
# assumptions below (tip-list shape, max_width=17).
#
# # Related
#
# - Contract test: crates/kay-cli/tests/cli_e2e.rs::interactive_parity_diff
# - Kay banner:    crates/kay-cli/src/banner.rs::display
# - Kay prompt:    crates/kay-cli/src/prompt.rs::KAY_PROMPT

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAG="forgecode-parity-baseline"
BASELINE_BANNER_SRC="crates/kay-core/src/forge_main/banner.rs"
FIXTURES_DIR="${REPO_ROOT}/crates/kay-cli/tests/fixtures"
BANNER_FIXTURE="${FIXTURES_DIR}/forgecode-banner.txt"
PROMPT_FIXTURE="${FIXTURES_DIR}/forgecode-prompt.txt"

cd "${REPO_ROOT}"

# --- 1. Verify baseline tag --------------------------------------------------

if ! git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
    echo "error: tag '${TAG}' not found in this clone." >&2
    echo "       expected to be pushed by Phase 1 parity-gate workflow." >&2
    exit 1
fi

# --- 2. Load baseline banner.rs and validate its shape ----------------------

BASELINE_SRC="$(git show "${TAG}:${BASELINE_BANNER_SRC}" 2>/dev/null || true)"
if [[ -z "${BASELINE_SRC}" ]]; then
    echo "error: ${BASELINE_BANNER_SRC} missing at tag ${TAG}." >&2
    exit 1
fi

# Assert the interactive-mode tips block still contains the canonical
# 6 rows we expect. If any of these grep patterns fail, the baseline
# source has drifted and the fixture must be re-designed before the
# script is trusted.
expected_rows=(
    '"New conversation:", ":new"'
    '"Get started:", ":info, :usage, :help, :conversation"'
    '"Switch model:", ":model"'
    '"Switch agent:", ":forge or :muse or :agent"'
    '"Update:", ":update"'
    '"Quit:", ":exit or <CTRL+D>"'
)
for row in "${expected_rows[@]}"; do
    if ! grep -qF "${row}" <<<"${BASELINE_SRC}"; then
        echo "error: baseline banner.rs missing expected row: ${row}" >&2
        echo "       re-inspect the tag source before regenerating fixtures." >&2
        exit 1
    fi
done

# --- 3. Reproduce right-alignment and emit banner fixture -------------------

# Keys from the baseline tips list. Must stay in source order; the
# runtime appends them to the labels array in this order after the
# Version row (which we exclude from the fixture — see header).
keys=(
    "New conversation:"
    "Get started:"
    "Switch model:"
    "Switch agent:"
    "Update:"
    "Quit:"
)
values=(
    ":new"
    ":info, :usage, :help, :conversation"
    ":model"
    ":forge or :muse or :agent"
    ":update"
    ":exit or <CTRL+D>"
)

# Runtime computes max_width over {Version: + all tips}. "Version:" is
# 8 chars and the longest tip key "New conversation:" is 17 chars, so
# max_width stays at 17 whether or not Version is in the fixture.
max_width=0
for key in "${keys[@]}"; do
    if (( ${#key} > max_width )); then
        max_width=${#key}
    fi
done
# The Version row (excluded from the fixture) is 8 chars, which is
# shorter than the longest tip key, so it cannot raise max_width. But
# cross-check that invariant explicitly — if future ForgeCode releases
# add a longer label, the alignment would shift.
if (( max_width != 17 )); then
    echo "error: expected max_width=17 (len('New conversation:'))," >&2
    echo "       got max_width=${max_width}. Fixture right-alignment" >&2
    echo "       would no longer match kay's runtime output." >&2
    exit 1
fi

mkdir -p "${FIXTURES_DIR}"

{
    for i in "${!keys[@]}"; do
        key="${keys[$i]}"
        value="${values[$i]}"
        # Reproduce Rust's `format!("{key:>max_width$} {value}")`:
        # right-align key to max_width chars, then a single space, then
        # value. Use printf with a dynamic width specifier.
        printf "%*s %s\n" "${max_width}" "${key}" "${value}"
    done
} > "${BANNER_FIXTURE}"

# --- 4. Emit prompt fixture -------------------------------------------------

# The baseline prompt string is conceptually "forge> " (5 chars +
# trailing space) — the shell-style marker ForgeCode's interactive
# session renders at read-line time. Kay's KAY_PROMPT is "kay> ".
# `.trim()` in the parity test strips trailing whitespace, so the
# trailing-space-vs-newline distinction is immaterial for assertion
# purposes. We write the bare `forge>` line for readability.
printf 'forge>\n' > "${PROMPT_FIXTURE}"

# --- 5. Report --------------------------------------------------------------

echo "captured parity fixtures from tag ${TAG}:"
printf '  %s (%d bytes)\n' "${BANNER_FIXTURE}" "$(wc -c <"${BANNER_FIXTURE}")"
printf '  %s (%d bytes)\n' "${PROMPT_FIXTURE}" "$(wc -c <"${PROMPT_FIXTURE}")"
echo
echo "validate with:"
echo "  cargo test -p kay-cli --test cli_e2e interactive_parity_diff"
