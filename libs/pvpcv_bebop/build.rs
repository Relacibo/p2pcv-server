use std::{fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bebop.json");
    println!("cargo:rerun-if-changed=schemas");

    // Regenerate from .bop schemas if bebopc is available.
    let status = Command::new("bebopc").args(["-c", "bebop.json"]).status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!("bebopc exited with status {s}"),
        Err(e) => eprintln!("cargo:warning=bebopc not found, using pre-generated file ({e})"),
    }

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
