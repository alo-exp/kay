## Summary

<!-- Short description of the change. Link related issues. -->

## Checklist

- [ ] I have run `cargo fmt -p <non-kay-core crates>` and `cargo clippy --workspace --exclude kay-core --all-targets -- -D warnings` locally (kay-core deferred to Phase 2 per `.planning/phases/01-fork-governance-infrastructure/VERIFICATION.md §SC-4`; see CONTRIBUTING.md §Pull Request Process for the full fmt command)
- [ ] Every commit is signed off (`Signed-off-by: Name <email>`) — the DCO job will fail otherwise
- [ ] I have not had exposure to the `@anthropic-ai/claude-code` v2.1.88 leak (2026-03-31); this PR contains no leak-derived code (clean-room attestation per CONTRIBUTING.md)
- [ ] I have updated tests where behavior changes
- [ ] I have updated docs (README, CONTRIBUTING, SECURITY) if the public contract changes

## Related Requirements

<!-- Reference REQ-IDs from .planning/REQUIREMENTS.md (e.g., GOV-04, WS-01). -->
