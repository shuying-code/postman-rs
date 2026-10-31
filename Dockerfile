# syntax=docker/dockerfile:1
# Slim, reproducible image for the pmrs CLI.
# Building only the `pmrs` binary keeps the image lean (the heavy GUI deps
# such as eframe/egui are not reachable from this target and are skipped).
FROM rust:1-slim AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin pmrs

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/pmrs /usr/local/bin/pmrs
ENTRYPOINT ["pmrs"]
CMD ["--help"]
