use std::{fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=generated");

    let src_dir = Path::new("src/generated");
    fs::create_dir_all(src_dir).expect("generated output dir should exist");
    for file in ["c2s.rs", "s2c.rs", "mod.rs"] {
        fs::copy(Path::new("generated").join(file), src_dir.join(file))
            .unwrap_or_else(|err| panic!("failed to copy generated/{file}: {err}"));
    }
}
