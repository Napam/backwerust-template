# syntax=docker/dockerfile:1

# glibc + bundled sqlite instead of musl/scratch: sqlx's sqlite feature compiles
# and links C (libsqlite3-sys), which needs a C toolchain in the builder and a
# compatible libc at runtime. The result is still a single self-contained binary
# with static assets and migrations embedded.
FROM rust:1.99-trixie AS builder

# bun drives the web build. Copied from the official image instead of piping a
# curl installer; bunx is the same binary under another name.
COPY --from=oven/bun:1-debian /usr/local/bin/bun /usr/local/bin/bun
RUN ln -s /usr/local/bin/bun /usr/local/bin/bunx

# task, so the build goes through the same Taskfile as local builds
RUN curl --fail --silent --show-error --location https://taskfile.dev/install.sh \
    | sh -s -- -b /usr/local/bin

WORKDIR /app

# Warm the dependency layers before copying sources.
COPY web/package.json web/bun.lock web/
RUN --mount=type=cache,target=/root/.bun/install/cache \
    cd web && bun install --frozen-lockfile

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch --locked

COPY . .

# bin/ is in .dockerignore (local build output), but task build copies into it
RUN mkdir -p bin

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    ENV=prod task build

FROM debian:trixie-slim

# ca-certificates only: sqlite is bundled into the binary, so the sqlite3 CLI
# isn't needed for the app to run.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/bin/app /app/bin/app

RUN mkdir -p /app/data

# 0.0.0.0 so the published port is reachable from the host; a bare `docker run`
# needs it (a container can't bind localhost). PORT comes from the Taskfile.
ENV HOST=0.0.0.0 \
    PORT=6900 \
    DB_MIGRATE_ON_START=true

ENTRYPOINT ["bin/app"]
