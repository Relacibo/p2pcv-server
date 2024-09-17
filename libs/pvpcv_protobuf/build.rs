use std::io::Result;
fn main() -> Result<()> {
    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build.compile_protos(
        &[
            "src/requests.proto",
            "src/responses.proto",
        ],
        &["src"],
    )?;
    Ok(())
}
