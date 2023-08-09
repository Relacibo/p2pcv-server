FROM rust:bookworm
WORKDIR /app
RUN apt update && DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends postgresql-client git
RUN cargo install diesel_cli@^2.1 --locked --no-default-features --features postgres

COPY /deployment/general/migrate/run.sh /usr/local/bin

RUN chmod +rx /usr/local/bin/run.sh

CMD [ "/bin/bash", "/usr/local/bin/run.sh" ]
