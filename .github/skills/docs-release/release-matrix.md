# Release Matrix — clickup-rs

## Platform Targets

| Target triple | OS | Arch | Artifact format | Runner |
|--------------|-----|------|----------------|--------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 | `.tar.gz` | `ubuntu-latest` |
| `aarch64-unknown-linux-gnu` | Linux | aarch64 | `.tar.gz` | `ubuntu-latest` (cross) |
| `x86_64-apple-darwin` | macOS | x86_64 | `.tar.gz` | `macos-latest` |
| `aarch64-apple-darwin` | macOS | aarch64 (Apple Silicon) | `.tar.gz` | `macos-latest` |
| `x86_64-pc-windows-msvc` | Windows | x86_64 | `.zip` | `windows-latest` |

## Build Strategy

- Native builds: macOS (both arches on `macos-latest`), Windows
- Cross-compiled: Linux aarch64 via `cross` tool
- All Linux/macOS artifacts: `tar.gz` containing `clickup-cli` and `clickup-tui` binaries
- Windows artifacts: `zip` containing `clickup-cli.exe` and `clickup-tui.exe`

## Checksums

Generate SHA256 checksums for every artifact:

```bash
sha256sum clickup-rs-*.tar.gz clickup-rs-*.zip > checksums.txt
```

Attach `checksums.txt` to the GitHub Release alongside all platform archives.

## Artifact Naming Convention

```
clickup-rs-{version}-{target}.{ext}
```

Examples:
- `clickup-rs-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `clickup-rs-v0.1.0-aarch64-apple-darwin.tar.gz`
- `clickup-rs-v0.1.0-x86_64-pc-windows-msvc.zip`
