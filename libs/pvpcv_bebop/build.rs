fn main() {
    let bebopc_path = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
    )
    .join("../../target/bebopc");
    bebop_tools::download_bebopc(bebopc_path);
    bebop_tools::build_schema_dir(
        "schemas",
        "src/generated",
        &bebop_tools::BuildConfig::default(),
    );
}
