# Phase 3 Deferred Items

Out-of-scope discoveries logged during Phase 3 execution. Not fixed inline per
execute-plan scope boundary (auto-fix only what the current task directly
caused).

## D-1: forge_domain lib-test E0432 (pre-existing)

**Discovered during:** Plan 03-01 Task 3 (`cargo check --workspace --all-targets`).

**Symptom:**

```
error[E0432]: unresolved import `forge_test_kit::json_fixture`
error: could not compile `forge_domain` (lib test) due to 1 previous error
```

**Repro on plain HEAD (no kay-tools changes):**

```bash
git stash -u && cargo check -p forge_domain --all-targets
# → same E0432; error is pre-existing workspace state
```

**Cause:** `forge_test_kit` no longer exports a `json_fixture` item, or
the import path in `forge_domain`'s test module drifted during the
Phase 2.5 sub-crate split. Neither Plan 03-01 nor any Phase 3 plan modifies
these files.

**Impact on 03-01:** `cargo check --workspace` (lib-only) passes green.
`cargo check --workspace --all-targets` fails on the pre-existing error.
Plan 03-01's verification gate is `cargo check -p kay-tools` + `cargo check
--workspace`, both green. The `--all-targets` run is a best-effort extra
not required by the plan's acceptance criteria.

**Owner:** Deferred to a forge_test_kit follow-up ticket (Phase 2.5 cleanup
or a dedicated "Phase 2.5 Wave 7" regression patch). Not blocking Phase 3.
