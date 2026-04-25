#!/bin/sh
set -e

ISSUE_URL=$(gh issue create \
  --repo alo-exp/silver-bullet \
  --title "doc-scheme.md compliance not enforced: CHANGELOG and ARCHITECTURE go stale across phases" \
  --label "bug" \
  --body "## Summary

SB's workflow skills (\`/silver:feature\`, \`superpowers:writing-plans\`, \`superpowers:executing-plans\`) do not explicitly instruct the agent to update \`docs/CHANGELOG.md\`, \`docs/ARCHITECTURE.md\`, \`docs/knowledge/YYYY-MM.md\`, and \`docs/lessons/YYYY-MM.md\` per \`docs/doc-scheme.md\`. The result is that documentation silently falls behind across phases, and the gap is only discovered when the user explicitly asks for a compliance audit.

## Observed failure

On the Kay project (alo-exp/kay), 8 phases shipped successfully (Phases 1–8) but only **Phase 1** has a CHANGELOG entry. Phases 2, 2.5, 3, 4, 5, 6, 7, and 8 are all missing. \`docs/ARCHITECTURE.md\` was left saying \"Phase 8 in progress\" after Phase 8 merged. \`docs/knowledge/INDEX.md\` referenced paths (\`docs/sessions/\`, \`docs/specs/\`) that don't exist.

The doc-scheme specifies clearly:

> **Every task** (finalization step): CHANGELOG.md, knowledge/YYYY-MM.md, lessons/YYYY-MM.md  
> **Architecture changes**: ARCHITECTURE.md (rewritten)

But none of the SB workflow skills — brainstorming, writing-plans, executing-plans, silver-feature — mention \`doc-scheme.md\` or instruct the agent to consult it at the finalization step.

## Root cause

The finalization step in \`superpowers:executing-plans\` (step 15) and the PATH 13 (SHIP) in \`silver-feature\` do not include an explicit \"update docs per doc-scheme.md\" gate. The agent only updates docs when explicitly asked, because nothing in the prompt chain surfaces the obligation.

## Proposed fix

1. **\`superpowers:executing-plans\`** — Add an explicit finalization checklist item at step 15:
   > Check \`docs/doc-scheme.md\` (if it exists). For every completed task:
   > - Append to \`docs/CHANGELOG.md\` (one entry, newest-first format)
   > - Append to \`docs/knowledge/YYYY-MM.md\` if architectural patterns, gotchas, or key decisions were encountered
   > - Append to \`docs/lessons/YYYY-MM.md\` if portable lessons were learned
   > - If this task changed the architecture, rewrite \`docs/ARCHITECTURE.md\` §Current State

2. **\`silver-feature\` PATH 13 (SHIP)** — Before the PR is raised, gate on doc compliance:
   > If \`docs/doc-scheme.md\` exists, verify that \`docs/CHANGELOG.md\` has an entry for this phase and \`docs/ARCHITECTURE.md\` reflects the completed state. If not, write the missing entries before merging.

3. **\`superpowers:writing-plans\`** — Include a reminder in the plan header template:
   > Documentation: update docs per \`docs/doc-scheme.md\` at finalization (CHANGELOG, ARCHITECTURE, knowledge/, lessons/)

## Impact

Without enforcement, every phase that ships via \`/silver:feature\` will silently accumulate doc debt, making the knowledge/lessons/CHANGELOG corpus stale by the time the user notices. The architecture doc becomes misleading (e.g., says a phase is \"in progress\" after it shipped). The CHANGELOG provides zero historical record for future contributors.

## Priority

High — affects every project using \`/silver:feature\` that has a \`docs/doc-scheme.md\`.
" 2>&1)

echo "ISSUE_URL=$ISSUE_URL"

ITEM_ID=$(gh project item-add 4 --owner alo-exp --url "$ISSUE_URL" --format json | jq -r .id 2>&1)
echo "ITEM_ID=$ITEM_ID"

gh project item-edit \
  --project-id PVT_kwDOA5OQY84BU8tb \
  --id "$ITEM_ID" \
  --field-id PVTSSF_lADOA5OQY84BU8tbzhMcRXE \
  --single-select-option-id 7e62dc72 2>&1

echo "BACKLOG_OK"
