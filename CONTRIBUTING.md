# Contributing to Kay

Thank you for considering a contribution. Kay is Apache-2.0 licensed and uses
the Developer Certificate of Origin (DCO) to track contributor provenance.

## Developer Certificate of Origin (DCO)

Every commit must be signed off to certify the Developer Certificate of Origin
(https://developercertificate.org/, v1.1). This is done by adding a trailer:

    Signed-off-by: Your Name <your.email@example.com>

You can add it automatically via:

    git commit -s -m "feat: add widget"

Fix an un-signed-off commit with:

    git commit --amend -s        # last commit
    git rebase --signoff HEAD~3  # last three

By signing off, you assert all four clauses of DCO v1.1 — that you wrote the
contribution (or have the right to submit it), under the project's license,
and consent to the contribution being public.

## Clean-Room Attestation

By signing off, I confirm I have not had exposure to leaked Claude Code source
code (`@anthropic-ai/claude-code` v2.1.88 source map leak, 2026-03-31) and that
this contribution contains no code derived from that leak.

This attestation is required because Kay competes in the agentic-coding space
where the 2026-03-31 leak contaminated a large number of derived projects.
If you have been exposed to the leaked source, please do not contribute to
areas of Kay that could resemble the leaked code — raise the concern on the
PR and the maintainers will route review accordingly.

## Pull Request Process

1. Open an issue first for non-trivial changes.
2. One PR per logical change. Keep PRs focused.
3. Before pushing, run fmt and clippy against every crate except `kay-core`
   (which is temporarily excluded until Phase 2's structural integration of the
   imported ForgeCode source — see
   `.planning/phases/01-fork-governance-infrastructure/VERIFICATION.md §SC-4`):

   ```
   cargo fmt -p kay-cli -p kay-provider-openrouter -p kay-sandbox-linux \
             -p kay-sandbox-macos -p kay-sandbox-windows -p kay-tauri -p kay-tui
   cargo clippy --workspace --exclude kay-core --all-targets -- -D warnings
   ```

   Note: `cargo fmt --all` will fail on `kay-core` (E0583 from forge_*/lib.rs
   naming) — use the explicit per-package list above instead, which matches
   what CI runs. See `.planning/phases/01-fork-governance-infrastructure/VERIFICATION.md §SC-4`
   for the full rationale.
4. CI must pass (DCO + lint + tri-OS tests + cargo-deny + cargo-audit).
5. All commits in a PR must carry `Signed-off-by:`. The DCO job fails the PR
   otherwise.
6. Maintainers review under the clean-room policy. PRs that claim clean-room
   provenance will be fast-tracked; PRs with ambiguous provenance may be
   routed through a second reviewer.

## Style

- Rust: project `rustfmt.toml` + `clippy -- -D warnings`.
- TypeScript (when UI lands in Phase 9+): project `biome` config.
- Commit messages: conventional commits (`feat:`, `fix:`, `docs:`, `chore:`,
  `refactor:`, `test:`, `ci:`) + a one-line summary, body if useful, plus
  `Signed-off-by:`.

## Reporting Security Issues

Please see SECURITY.md. Do not open public issues for security concerns.

## Code of Conduct

Kay follows the Contributor Covenant v2.1. See CODE_OF_CONDUCT.md.
