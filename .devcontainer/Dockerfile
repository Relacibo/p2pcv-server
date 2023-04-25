# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

RUN sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld postgresql-client-15 npm \
   && apt-get autoremove -y && apt-get clean -y

RUN npm install --global yarn

USER vscode
RUN rustup default nightly
RUN cargo install diesel_cli@^2.0 --no-default-features --features postgres

USER root
