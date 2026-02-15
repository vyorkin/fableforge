FROM rust:slim AS builder

# Install nightly toolchain as required by rust-toolchain.toml
RUN rustup toolchain install nightly && rustup default nightly

# Install build dependencies for native TLS (required by teloxide)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies: copy manifests first, build a dummy to cache deps
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/fableforge-core/Cargo.toml crates/fableforge-core/Cargo.toml
COPY crates/fableforge-llm/Cargo.toml crates/fableforge-llm/Cargo.toml
COPY crates/fableforge-tg/Cargo.toml crates/fableforge-tg/Cargo.toml
COPY cli/fableforge/Cargo.toml cli/fableforge/Cargo.toml

RUN mkdir -p crates/fableforge-core/src crates/fableforge-llm/src crates/fableforge-tg/src cli/fableforge/src \
    && echo "fn main() {}" > cli/fableforge/src/main.rs \
    && echo "fn main() {}" > crates/fableforge-tg/src/main.rs \
    && touch crates/fableforge-core/src/lib.rs \
    && touch crates/fableforge-llm/src/lib.rs \
    && cargo build --release --workspace \
    && rm -rf crates cli

# Build actual source
COPY crates crates
COPY cli cli

# Touch source files so cargo knows they changed
RUN touch crates/fableforge-core/src/lib.rs \
    crates/fableforge-llm/src/lib.rs \
    crates/fableforge-tg/src/main.rs \
    cli/fableforge/src/main.rs \
    && cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fableforge /usr/local/bin/fableforge

ENTRYPOINT ["fableforge"]
