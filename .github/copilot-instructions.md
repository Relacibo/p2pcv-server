## Generated code — do not edit manually

The following files are **auto-generated** by `bebopc` and must **never** be edited directly:

- `libs/pvpcv_bebop/generated/mod.rs`
- `libs/pvpcv_bebop/src/generated/mod.rs` *(copied from above by build.rs at compile time)*

### How to regenerate

Edit the `.bop` schema files under `libs/pvpcv_bebop/schemas/protocols/schemas/`, then run from `libs/pvpcv_bebop/`:

```sh
bebopc -c bebop.json
```

**Note:** bebopc v3.2.3 has a bug — it does not write to `outFile` unless the file already exists with content. Workaround: the committed `generated/mod.rs` must always be present. After running `bebopc`, commit both `generated/mod.rs` and `src/generated/mod.rs` (or let `build.rs` handle the copy).

## SeaORM entities — do not edit manually

The files under `src/db/entities/` are **auto-generated** by `sea-orm-cli` and must **never** be edited directly.

### How to regenerate

1. Make sure the database is running (e.g. `docker compose up -d`)
2. Run: `just entities`

This reads the live schema (after migrations have been applied) and overwrites `src/db/entities/`.
Always run `just migrate` before `just entities` after adding a new migration.

## Conventions

### Enum values in the database and JSON bodies

Enum variants stored as text in PostgreSQL and serialized in JSON request/response bodies use **kebab-case** (e.g. `waiting`, `in-game`, `finished`). Use `#[serde(rename_all = "kebab-case")]` on all enums.

## API Documentation

Die gesamte API ist in `docs/api.md` dokumentiert. Bei Änderungen an Endpoints diese Datei aktualisieren.

### Dokumentationsformat

Jeder Endpoint hat ein **Schema** (TypeScript-Typen) und ein **Example** (konkretes JSON). Regeln:

- Typen werden als TypeScript geschrieben, z.B. `string | null`, `"waiting" | "in-game" | "finished"`
- Bei Enums **alle** Varianten auflisten
- IDs sind echte UUIDs im Beispiel, z.B. `"f47ac10b-58cc-4372-a567-0e02b2c3d479"`
- Timestamps als ISO 8601: `"2024-03-15T09:12:34Z"`
- `avatarHash` ist ein SHA-256 Hex-String (64 Zeichen)
- Im Example **niemals `null`** verwenden – stattdessen einen sinnvollen Wert zeigen
- Im Example **niemals `...`** oder Platzhalter wie `"string"` / `"uuid"` verwenden
