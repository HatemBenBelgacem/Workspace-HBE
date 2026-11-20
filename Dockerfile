# syntax=docker/dockerfile:1

# ---- Basis: Rust, Node, WebAssembly-Ziel, cargo-leptos und cargo-chef ----
FROM rust:1-bookworm AS chef

# Node für Tailwind und das DaisyUI-Plugin; Tailwind 4 braucht Node >= 20, Debian liefert nur 18.
COPY --from=node:22-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-leptos@0.3.11 cargo-chef@0.1.77 --no-confirm

WORKDIR /app

# ---- Planer: Rezept mit allen Abhängigkeiten aus Cargo.toml und Cargo.lock ----
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- Build: Server-Binary, WebAssembly und CSS mit cargo-leptos ----
FROM chef AS builder

# Abhängigkeiten vorab bauen. Diese Schichten bleiben im Cache, solange sich
# Cargo.toml und Cargo.lock nicht ändern. Profil und Features müssen genau denen
# entsprechen, mit denen cargo-leptos baut (siehe [package.metadata.leptos]).
# cargo-leptos baut das WebAssembly in einen eigenen Ordner (target/front).
# Leptos liest LEPTOS_OUTPUT_NAME beim Kompilieren; cargo-leptos setzt die Variable,
# deshalb muss sie hier schon gleich sein, sonst wird Leptos neu gebaut.
ENV LEPTOS_OUTPUT_NAME=workspace-hbe
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
        --no-default-features --features ssr
RUN cargo chef cook --profile wasm-release --recipe-path recipe.json \
        --target wasm32-unknown-unknown --target-dir target/front \
        --no-default-features --features hydrate

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
# Liste der Datei-Hashes; Leptos sucht sie im Ordner des Programms.
COPY --from=builder /app/target/release/hash.txt /app/hash.txt
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

ENV RUST_LOG=info \
    LEPTOS_ENV=PROD \
    LEPTOS_OUTPUT_NAME=workspace-hbe \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_HASH_FILES=true \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_RELOAD_PORT=3011

# Railway setzt PORT und DATABASE_URL; ohne PORT gilt LEPTOS_SITE_ADDR.
EXPOSE 8080
CMD ["/app/workspace-hbe"]
