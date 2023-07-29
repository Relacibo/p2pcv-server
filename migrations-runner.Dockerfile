FROM rust:slim-bookworm
RUN apt update && apt install postgresql-client
RUN cargo install diesel_cli@^2.1 --no-default-features --features postgres

COPY migrations .

CMD ["diesel", "migration", "run"]
