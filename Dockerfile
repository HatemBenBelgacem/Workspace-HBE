# syntax=docker/dockerfile:1

# ---- Build: Server-Binary, WebAssembly und CSS mit cargo-leptos ----
FROM rust:1-bookworm AS builder

# Node für Tailwind und das DaisyUI-Plugin; Tailwind 4 braucht Node >= 20, Debian liefert nur 18.
COPY --from=node:22-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-leptos@0.3.11 --no-confirm

WORKDIR /app
# Ohne package-lock.json: Der Lockfile entsteht auf macOS und enthält die native
# Linux-Variante von Tailwind (optionale Abhängigkeit) nicht.
COPY package.json ./
RUN npm install --ignore-scripts --no-audit --no-fund

COPY . .
RUN cargo leptos build --release

# ---- Laufzeit: nur Binary und statische Dateien ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/workspace-hbe /app/workspace-hbe
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

ENV RUST_LOG=info \
    LEPTOS_ENV=PROD \
    LEPTOS_OUTPUT_NAME=workspace-hbe \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_RELOAD_PORT=3011

# Railway setzt PORT und DATABASE_URL; ohne PORT gilt LEPTOS_SITE_ADDR.
EXPOSE 8080
CMD ["/app/workspace-hbe"]
