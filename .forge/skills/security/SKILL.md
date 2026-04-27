---
id: security
title: Security
description: Security review checklist for Rust and TypeScript code — input validation, credential handling, IPC boundaries, sandbox policy.
trigger: security, auth, credentials, validation, sanitize, inject, escape, sandbox
---

# Security

Apply before committing any code that touches auth, input, IPC, or external data.

## Input Validation
- Validate all IPC inputs at the Tauri command boundary (not deep in business logic)
- Use typed structs for deserialization — never `serde_json::Value` for user-controlled data without validation
- Reject inputs exceeding reasonable size limits before processing

## Credential Handling
- Never log API keys, tokens, or secrets — even at debug level
- Store secrets in environment variables or OS keychain; never hardcode or commit
- Redact credential fields when serializing structs for logging

## IPC Boundary (Tauri)
- All `#[tauri::command]` handlers validate inputs before touching system state
- Use `CancellationToken` for session teardown — never expose raw channel senders to frontend
- `AppState` fields are `DashMap` or `Mutex`-wrapped; no raw `HashMap` behind shared state

## Sandbox Policy
- Tool executions run under platform sandbox (`sandbox-exec` macOS, `landlock` Linux, Job Objects Windows)
- Never bypass sandbox for "convenience" — if a tool needs a resource, update the policy explicitly
- Log all sandbox violations as `AgentEvent::SandboxViolation`

## Dependency Hygiene
- Pin exact versions for RC/pre-release crates (e.g., `=2.0.0-rc.21`)
- Run `cargo audit` before releasing; no known vulnerabilities in direct dependencies
