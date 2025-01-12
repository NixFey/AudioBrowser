FROM lukemathwalker/cargo-chef:0.1.68-rust-1.84 AS base
WORKDIR /app

FROM base AS planner
COPY . .
RUN cargo chef prepare --bin AudioBrowser --recipe-path recipe.json

FROM base AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --bin AudioBrowser --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin AudioBrowser

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/AudioBrowser /usr/local/bin
COPY --from=builder /app/assets /app/assets
ENTRYPOINT ["/usr/local/bin/AudioBrowser"]
