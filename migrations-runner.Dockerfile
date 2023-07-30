FROM rust:slim-bookworm
RUN apt update && apt install postgresql-client
RUN cargo install diesel_cli@^2.1 --no-default-features --features postgres

COPY /deployment/base/migrations-runner/run.sh /usr/local/bin

WORKDIR /app

CMD [ "/usr/local/bin/run.sh" ]
