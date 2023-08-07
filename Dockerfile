FROM lukemathwalker/cargo-chef:latest-rust-1.71 AS chef
WORKDIR /app

FROM chef AS planner
COPY ./.git ./src ./Cargo.toml ./Cargo.lock ./rust-toolchain.toml ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY ./.git ./src ./Cargo.toml ./Cargo.lock ./rust-toolchain.toml ./
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/p2pcv-server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/p2pcv-server"]
