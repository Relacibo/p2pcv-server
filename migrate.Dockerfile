FROM rust:bookworm
RUN apt update && apt install postgresql-client && apt install git
RUN cargo install diesel_cli@^2.1 --no-default-features --features postgres

COPY /deployment/base/migrate/run.sh /usr/local/bin

WORKDIR /app

CMD [ "/usr/local/bin/run.sh" ]
