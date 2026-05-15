ARG RUST_VERSION=1.95.0
ARG CARGO_CHEF_VERSION=0.1.77
ARG DEBIAN_TAG=trixie-slim
FROM rust:${RUST_VERSION} AS chef
RUN cargo install cargo-chef@${CARGO_CHEF_VERSION} --locked
RUN apt update \
  && DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends \
  build-essential clang make cmake pkg-config \
  && apt-get autoremove -y && apt-get clean -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin app

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin app
COPY . .
RUN cargo build --release --bin app

FROM debian:${DEBIAN_TAG}
WORKDIR /app
RUN apt update && \
  DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends libssl3 ca-certificates \
  && apt-get autoremove -y && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /usr/local/bin
CMD ["/usr/local/bin/app", "migrate"]
