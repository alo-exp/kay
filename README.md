# Kay

Open-source terminal coding agent — a Rust fork of ForgeCode, hardened with Terminus-KIRA harness techniques, and delivered through a native Tauri desktop UI.

## Status

v0.6.0 (Phase 13 complete). Feature parity with Forge achieved. 
- Live API: MiniMax, OpenRouter
- Commands: build, check, fmt, clippy, test, review, session
- Tools: fs_search, shell, fetch, task, patch
- Sessions: list, load, delete, rewind
- Markdown rendering with ANSI colors
- Token budget management, retry logic, JSON repair, diff highlighting

Phase 12 (TB 2.0 Submission + v1 Hardening) planned. See `.planning/ROADMAP.md`.

## Acknowledgments

Kay is a fork of [ForgeCode](https://github.com/antinomyhq/forgecode)
(Copyright 2025 Tailcall, Apache-2.0) — Tailcall's ForgeCode is the
base harness Kay builds on. See `NOTICE` and `ATTRIBUTIONS.md` for
attribution details.

Kay's harness techniques — multi-perspective verification, marker-based
shell polling, schema hardening, tool registry design — are informed by
[Terminus-KIRA](https://github.com/terminus-ai/KIRA), whose published
approaches Kay re-implements from specification in clean-room fashion.

## Contributing

See `CONTRIBUTING.md` for the DCO + clean-room attestation workflow.

## Security

See `SECURITY.md` for vulnerability disclosure.

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
