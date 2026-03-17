---
applyTo: "Dockerfile,docker-compose.yml,docker-compose*.yml"
---

# Docker Instructions

## Dockerfile
- Use multi-stage builds: builder stage with `rust:1.85-slim`, runtime stage with `debian:bookworm-slim`
- Install `ca-certificates` in runtime stage (needed for HTTPS/TLS)
- Build both `clickup-cli` and `clickup-tui` binaries in release mode
- Copy only the final binaries to runtime stage
- Use non-root user in runtime stage

## docker-compose.yml
- Service name: `clickup`
- Mount `~/.config/clickup-rs` as a volume for config persistence
- Set `CLICKUP_LOG` environment variable to `info` by default
