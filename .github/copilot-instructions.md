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
