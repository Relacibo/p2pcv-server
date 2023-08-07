FROM lukemathwalker/cargo-chef:latest-rust-1.71-bookworm AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.71-bookworm AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/p2pcv-server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/p2pcv-server"]
