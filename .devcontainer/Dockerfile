# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
FROM mcr.microsoft.com/devcontainers/rust:latest

RUN sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld postgresql-client-16 npm redis-server \ 
   && apt-get autoremove -y && apt-get clean -y

RUN npm install --global yarn
RUN npm install --global dotenv-vault

# http://google.github.io/proto-lens/installing-protoc.html
ENV PROTOC_VERSION 3.14.0
ENV PROTOC_ZIP protoc-${PROTOC_VERSION}-linux-x86_64.zip
RUN curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/${PROTOC_ZIP}
RUN unzip -o $PROTOC_ZIP -d /usr/local bin/protoc && chmod +x /usr/local/bin/protoc
RUN unzip -o $PROTOC_ZIP -d /usr/local 'include/*' && chmod -R +r /usr/local/include/google
RUN rm -f $PROTOC_ZIP

USER vscode
RUN cargo install diesel_cli@^2.2 --no-default-features --features postgres
# RUN cargo install diesel_cli_ext@^0.3

USER root
