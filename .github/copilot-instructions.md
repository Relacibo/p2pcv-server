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

The complete API is documented in `docs/api.md`. Update this file when adding or changing endpoints.

### Language

All code comments, `docs/api.md`, and other developer documentation must be written in **English**.

### Documentation format

Each endpoint has a **Schema** (TypeScript types) and an **Example** (concrete JSON). Rules:

- Types are written as TypeScript, e.g. `string | null`, `"waiting" | "in-game" | "finished"`
- Use `field?: Type` for fields that may be **absent** from the JSON (e.g. optional request body fields, `skip_serializing_if`)
- Use `field: Type | null` for fields that are **always present** in the JSON but can be null
- For enums, list **all** variants
- IDs must be real UUIDs in examples, e.g. `"f47ac10b-58cc-4372-a567-0e02b2c3d479"`
- Timestamps as ISO 8601: `"2024-03-15T09:12:34Z"`
- `avatarHash` is a SHA-256 hex string (64 characters)
- **Never use `null`** in examples for a single-object response – show a meaningful value instead
- For array responses, if a field is nullable, the **first object** shows a real value; the **second object** may use `null` to demonstrate the nullable case
- **Never use `...`** or placeholders like `"string"` / `"uuid"` in examples
