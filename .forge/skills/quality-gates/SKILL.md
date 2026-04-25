---
id: quality-gates
title: Quality Gates
description: Enforce code quality standards before committing changes — modularity, error handling, naming, and test coverage.
trigger: quality, gates, review, checklist, standards
---

# Quality Gates

Apply these checks before marking any task complete.

## Modularity
- Each file has one clear purpose; split files >300 lines if they mix concerns
- Public API surface is minimal — unexported functions are preferred unless cross-crate use is needed
- No circular dependencies between crates

## Error Handling
- Use `Result<T, E>` for fallible operations; never `.unwrap()` in library code
- Propagate errors with `?`; add context with `.map_err(|e| format!("context: {e}"))`
- Never silently swallow errors

## Naming
- Rust: `snake_case` functions/variables, `PascalCase` types/traits, `SCREAMING_SNAKE` constants
- TypeScript: `camelCase` functions/variables, `PascalCase` types/components
- Names describe intent; avoid abbreviations except well-known ones (`tx`, `rx`, `buf`)

## Tests
- Every public function has at least one unit test
- Tests are in the same file (`#[cfg(test)] mod tests`) for unit tests, `tests/` for integration
- Test names describe what they verify: `test_flush_drains_buffer_on_channel_close`

## Commit Hygiene
- Atomic commits: one logical change per commit
- Commit message: imperative mood, <72 chars subject, body explains WHY
- Always append: `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`
