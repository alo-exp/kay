# Security Policy

## Supported Versions

Kay is pre-1.0 software. Only the most recent published tag on `main` is
supported. Security fixes are landed on `main` and released as new patch
versions.

| Version      | Supported |
|--------------|-----------|
| main / HEAD  | Yes       |
| Older tags   | No        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for a security concern.**

Report security issues via GitHub's private Security Advisory flow:

1. Go to https://github.com/alo-exp/kay/security/advisories
2. Click "Report a vulnerability"
3. Fill in the details

If you cannot use GitHub Security Advisories, email
security@kay.dev (preferred) with:
- A clear description of the vulnerability.
- Reproduction steps or a proof-of-concept.
- The version(s) affected.
- Your preferred name / handle for acknowledgment in the advisory.

## Response SLA

Kay is solo-maintained pre-1.0. Expect:
- **Acknowledgment** within 72 hours.
- **Assessment + triage** within 7 days.
- **Fix or mitigation plan** within 30 days for critical issues,
  90 days for lower-severity issues.

These SLAs will tighten as Kay matures. SLAs are best-effort, not guaranteed;
please be patient.

## Coordinated Disclosure

We follow coordinated disclosure:
- 90-day private disclosure window by default.
- Advisory published simultaneously with the fix release.
- Reporter credited in the advisory unless anonymity is requested.

## Release Signing

All release tags from v0.1.0 onward are GPG- or SSH-signed. Public signing
keys are published at https://github.com/alo-exp/kay/tree/main/docs/signing-keys.
Verify a release with:

    git tag -v v0.1.0

If `git tag -v` reports "no signature found" or a verification failure,
do not trust the release. Contact security@kay.dev.

## Dependency Hygiene

- `cargo-audit` runs on every PR and nightly against the RustSec Advisory
  Database.
- `cargo-deny` enforces a license allowlist (no GPL/AGPL/LGPL transitively)
  and blocks known-vulnerable crates.
- Dependency updates require a passing CI run.

## Attribution

Kay's security process is informed by the OpenSSF oss-vulnerability-guide
(https://github.com/ossf/oss-vulnerability-guide).

**Note:** This policy is pre-1.0 and maintainer-reviewed only. A formal legal review of the clean-room attestation clause in `CONTRIBUTING.md` is planned before Kay reaches its 0.10 series or accepts its first external contribution — whichever comes first.
