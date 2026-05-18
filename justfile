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
        --with-serde both \
        --model-extra-attributes 'serde(rename_all = "camelCase"), SKIP_SERIALIZING_NONE_PLACEHOLDER'
    sed -i '/SKIP_SERIALIZING_NONE_PLACEHOLDER/d' src/db/entities/*.rs
    sed -i '/#\[derive(/i #[serde_with::skip_serializing_none]' src/db/entities/*.rs

# Regenerate bebop types from schemas
bebop:
    cargo build -p p2pcv-bebop
