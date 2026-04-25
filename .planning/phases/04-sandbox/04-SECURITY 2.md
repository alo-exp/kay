---
phase: 4
date: 2026-04-21
status: passed
---

# Phase 4 Security Review

## Threat Model

| ID | Threat | Mitigation | Status |
|----|--------|-----------|--------|
| T-4-01 | Escape via write outside project root | macOS: SBPL `(deny default)` + `(allow file-write* (subpath "<root>"))`; Linux: Landlock filesystem ruleset; Windows: Job Object filesystem ACL | ✅ Kernel-enforced |
| T-4-02 | Escape via symlink (Linux Landlock) | Known: Landlock is path-based — denied subtree denied even via symlink. Advisory documented here. Pre-flight check does NOT follow symlinks. | ⚠️ Advisory |
| T-4-03 | Network exfiltration to non-allowlisted host | Pre-flight: `SandboxPolicy.allows_net()`; Kernel: macOS SBPL `(allow network-outbound (remote tcp "*:443"))` for allowlisted port; Linux: seccomp BPF + Landlock | ✅ Defense-in-depth |
| T-4-04 | `sandbox-exec` deprecation (macOS 15) | Functional through 15.x. `KaySandboxMacos::new()` returns `SandboxError::BackendUnavailable` if binary absent. Phase 11 monitoring planned. | ✅ Handled |
| T-4-05 | Grandchild escape via orphan process (Windows) | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` kills all Job Object members on Kay exit. R-4 closed. | ✅ R-4 closed |
| T-4-06 | SBPL profile injection (user-controlled content in profile) | SBPL profile built from `SandboxPolicy` struct fields (typed). No user-controlled string interpolation in profile paths. | ✅ Safe |
| T-4-07 | SandboxViolation re-injected into model context | QG-C4 documented in events.rs + 04-CONTEXT.md: `SandboxViolation` MUST NOT be serialized into model message history. Phase 5 planning constraint. | ✅ Documented |
| T-4-08 | TOCTOU between pre-flight and kernel enforcement | Pre-flight checks are advisory; kernel enforcement is authoritative. Race is acceptable — the kernel will deny the attempt regardless. | ✅ By design |
| T-4-09 | Landlock degradation exposes attack surface | On kernel < 5.13, `tracing::warn!` emitted, seccomp-only mode used. This is a weaker posture but not a silent failure. | ⚠️ Advisory |
| T-4-10 | Read of sensitive paths (`.ssh`, `.aws`, etc.) | `SandboxPolicy.read_deny_list` defaults include `~/.aws`, `~/.ssh`, `~/.gnupg`, `~/.gnupg2`. Pre-flight enforcement via `check_fs_read()`. | ✅ Pre-flight |

---

## Non-Negotiable Compliance

| NN | Requirement | Status |
|----|------------|--------|
| NN#1 | ForgeCode parity gate | N/A — no parity-sensitive changes in Phase 4 |
| NN#2 | ED25519 signed release tags | Phase 4 tag v0.2.0 will be signed at ship time |
| NN#3 | DCO on every commit | All Phase 4 commits carry `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` |
| NN#4 | Clean-room (no TS-derived structure) | Phase 4 is pure Rust OS syscall work |
| NN#5 | Single binary, no externalBin | `sandbox-exec` is a system binary; no `externalBin` entry added |
| NN#6 | OpenRouter allowlist | `SandboxPolicy.default_for_project()` includes `{host: "openrouter.ai", port: 443}` |
| NN#7 | Schema hardening | No new tool schemas in Phase 4 |

---

## Advisories

**T-4-02 (Landlock symlink):** Linux Landlock operates on path components, not inodes. A symlink pointing into a denied subtree is itself denied — but a symlink that resolves *through* the allowed subtree and exits it is also denied at the exit point. Callers should not assume `allows_read(symlink_path)` is sufficient; the real path must be checked. Document at Phase 5 integration.

**T-4-09 (Landlock degradation):** On kernel < 5.13, the seccomp-only mode has no filesystem path isolation — only syscall filtering. Users on old kernels should be warned at Kay startup. Phase 5 planning constraint: emit `AgentEvent::Warning` if `KaySandboxLinux.landlock_available() == false`.

---

## Phase 5 Constraints Carried Forward

1. **QG-C4:** `AgentEvent::SandboxViolation` MUST NOT be serialized into model message history.
2. **T-4-09:** Emit `AgentEvent::Warning` if Landlock unavailable (kernel < 5.13).
3. **QG-C1:** Add criterion micro-bench for macOS spawn latency < 5ms before Phase 5 ships.
