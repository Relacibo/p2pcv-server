# p2pcv-server justfile

set dotenv-load := true

default:
    @just --list

# Run SeaORM migrations
migrate:
    cargo run -p migration

# Generate a new migration: just migration create_games
migration name:
    sea-orm-cli migrate generate {{ name }} --migration-dir migration/src

# Regenerate SeaORM entities from the running DB (requires DB to be up)
entities:
    sea-orm-cli generate entity \
        --database-url "$DATABASE_URL" \
        --output-dir src/db/entities \
        --with-serde both

# Regenerate bebop types from schemas
bebop:
    cargo build -p p2pcv-bebop
