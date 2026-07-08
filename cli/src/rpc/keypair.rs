use std::path::PathBuf;

use anyhow::Result;
use solana_sdk::signature::{read_keypair_file, Keypair};

use crate::config::DEFAULT_KEYPAIR_PATH;

pub fn resolve_keypair_path(explicit: Option<&str>) -> PathBuf {
    expand_tilde(explicit.unwrap_or(DEFAULT_KEYPAIR_PATH))
}

pub fn load_keypair(explicit: Option<&str>) -> Result<Keypair> {
    let path = resolve_keypair_path(explicit);
    read_keypair_file(&path).map_err(|e| {
        anyhow::anyhow!("load keypair {}: {e}", path.display())
    })
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_path_uses_tilde_expansion() {
        let path = resolve_keypair_path(None);
        assert!(!path.to_string_lossy().starts_with("~/"));
    }
}
