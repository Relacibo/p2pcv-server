FROM rust:slim-bookworm
RUN apt update && DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends postgresql-client git
RUN cargo install diesel_cli@^2.1 --no-default-features --features postgres

COPY /deployment/base/migrate/run.sh /usr/local/bin

WORKDIR /app

CMD [ "/usr/local/bin/run.sh" ]
