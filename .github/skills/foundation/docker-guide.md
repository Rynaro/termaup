# Docker Guide — clickup-rs

## Dockerfile

Use a multi-stage build to minimize image size:

**Builder stage:**
- Base image: `rust:1.85-slim`
- Copy workspace `Cargo.toml`, `Cargo.lock`, and all member crate manifests first (layer caching)
- Build both `clickup-cli` and `clickup-tui` binaries in `--release` mode

**Runtime stage:**
- Base image: `debian:bookworm-slim`
- Install `ca-certificates` (required for HTTPS/TLS with rustls)
- Copy only the final binaries from builder stage
- Create and switch to a non-root user (`clickup:clickup`)
- Set `ENTRYPOINT` to `clickup-cli` binary

## docker-compose.yml

```yaml
services:
  clickup:
    build: .
    volumes:
      - ~/.config/clickup-rs:/home/clickup/.config/clickup-rs
    environment:
      CLICKUP_LOG: info
```

- Service name: `clickup`
- Mount `~/.config/clickup-rs` as a volume for config and token persistence
- Set `CLICKUP_LOG` environment variable to `info` by default
