ARG RUST_VERSION=1.80
ARG CARGO_CHEF_VERSION=0.1.67
ARG DEBIAN_TAG=bookworm-20240904-slim
FROM rust:${RUST_VERSION} AS chef
RUN cargo install cargo-chef@${CARGO_CHEF_VERSION} --locked
RUN apt update \
  && DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends \
  install build-essential clang make cmake pkg-config \
  && apt-get autoremove -y && apt-get clean -y
# http://google.github.io/proto-lens/installing-protoc.html
ENV PROTOC_VERSION 3.14.0
ENV PROTOC_ZIP protoc-${PROTOC_VERSION}-linux-x86_64.zip
RUN curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/${PROTOC_ZIP}
RUN unzip -o $PROTOC_ZIP -d /usr/local bin/protoc && chmod +x /usr/local/bin/protoc
RUN unzip -o $PROTOC_ZIP -d /usr/local 'include/*' && chmod -R +r /usr/local/include/google
RUN rm -f $PROTOC_ZIP
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

FROM debian:${DEBIAN_TAG}
WORKDIR /app
RUN apt update && \
  DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends libssl-dev cmake \
  && apt-get autoremove -y && apt-get clean -y
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/app"]

EXPOSE 8000
