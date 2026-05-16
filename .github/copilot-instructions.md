# GitHub Copilot Instructions

## Generated code — do not edit manually

The following files are **auto-generated** by `bebopc` and must **never** be edited directly:

- `libs/pvpcv_bebop/generated/mod.rs`
- `libs/pvpcv_bebop/src/generated/mod.rs`

To regenerate, run from the repo root:

```sh
cd libs/pvpcv_bebop && bebopc -c bebop.json
```

`build.rs` then copies `generated/mod.rs` → `src/generated/mod.rs` at compile time.
To change the generated code, edit the `.bop` schema files under
`libs/pvpcv_bebop/schemas/protocols/schemas/` and re-run `bebopc`.
