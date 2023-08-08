FROM lukemathwalker/cargo-chef:latest-rust-1.71 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin app

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --bin app
# Build application
COPY . .
RUN cargo build --release --bin app

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && apt-get install -y --no-install-recommends ca-certificates
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/app"]

EXPOSE 8000
