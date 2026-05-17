use std::{fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=generated/mod.rs");

    // NOTE: bebopc v3.2.3 does not write to outFile automatically.
    // Run `bebopc -c bebop.json` manually after schema changes, then commit generated/mod.rs.
    // If bebopc is available we still try, in case a future version fixes this.
    let _ = Command::new("bebopc").args(["-c", "bebop.json"]).status();

    let src_dir = Path::new("src/generated");
    fs::create_dir_all(src_dir).expect("generated output dir should exist");
    let file = "mod.rs";
    fs::copy(Path::new("generated").join(file), src_dir.join(file))
        .unwrap_or_else(|err| panic!("failed to copy generated/{file}: {err}"));
    // Remove old split files if they still exist from a previous codegen run.
    for file in ["c2s.rs", "s2c.rs"] {
        let _ = fs::remove_file(src_dir.join(file));
    }
}
