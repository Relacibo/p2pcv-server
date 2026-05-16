use std::{env, fs};

/// Reads a secret value: if `{name}_FILE` is set, reads from that file path.
/// Otherwise falls back to reading `{name}` directly as an env var.
pub fn read_secret(name: &str) -> String {
    let file_var = format!("{name}_FILE");
    if let Ok(path) = env::var(&file_var) {
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Could not read secret file {path} ({file_var}): {e}"))
            .trim_end_matches('\n')
            .to_string()
    } else {
        env::var(name).unwrap_or_else(|_| panic!("{name} (or {file_var}) needs to be set!"))
    }
}
