FROM rust:1.88-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --workspace --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home appuser
USER appuser

ENV HOME=/home/appuser

COPY --from=builder /app/target/release/clickup-cli /usr/local/bin/clickup-cli
COPY --from=builder /app/target/release/clickup-tui /usr/local/bin/clickup-tui

# Default to CLI; override entrypoint for TUI.
ENTRYPOINT ["clickup-cli"]
CMD []
