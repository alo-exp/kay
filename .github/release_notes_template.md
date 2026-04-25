# Kay v{{ version }}

Release date: {{ date }}

## What's New

See [CHANGELOG.md](CHANGELOG.md) for full release notes.

## Installation

### CLI
```bash
cargo install kay
```

### TUI
```bash
cargo install kay-tui
```

### Desktop App
Download the bundled app for your platform:
- **macOS**: Kay-*.app (arm64 + x64)
- **Windows**: Kay-*.msi
- **Linux**: Kay-*.tar.gz

## Verification

All artifacts are signed. Verify the SHA attestations:

```bash
# Download SHA256SUMS.txt and SHA256SUMS.minisig from the release
sha256sum -c SHA256SUMS.txt
minisign -V -p kay-2026.kay -x SHA256SUMS.txt.minisig
```

## Source

https://github.com/alo-exp/kay
