# Release Signing Keys

This directory publishes the GPG and/or SSH public keys used to sign Kay
release tags. Phase 1 ships no tags; the first signed tag lands in a later
phase (Phase 11 per the deferred procurement schedule in
`.planning/phases/01-fork-governance-infrastructure/01-CONTEXT.md §User
Amendments` — D-OP-04).

When a key is added here:

- Filename convention: `{handle}-{algorithm}.pub` (e.g. `shafqat-ssh.pub`,
  `shafqat-gpg.asc`).
- Keys are rotated annually or on compromise (per D-15 in
  `.planning/phases/01-fork-governance-infrastructure/01-CONTEXT.md`).
- SECURITY.md cross-references this directory.

Verify a release tag with:

    git tag -v vX.Y.Z
