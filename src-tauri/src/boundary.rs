//! Fail-closed egress: Desk may spawn only a local Codex binary.

use std::path::{Path, PathBuf};

const ALLOWED_NAMES: &[&str] = &["codex", "codex.exe", "codex.cmd"];

pub fn assert_local_codex(path: &Path) -> Result<PathBuf, String> {
    let raw = path.to_string_lossy();
    if raw.contains("://") || raw.starts_with(r"\\") {
        return Err(
            "Refusing a remote Codex path. Desk may spawn only a local `codex` binary.".into(),
        );
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !ALLOWED_NAMES.contains(&name.as_str()) {
        return Err(format!(
            "Refusing binary `{name}`. Allowlist is local Codex only (codex / codex.exe / codex.cmd)."
        ));
    }
    if !path.is_file() {
        return Err(format!("Codex binary is not a local file: {}", path.display()));
    }
    Ok(path.to_path_buf())
}

pub fn is_cleartext_url(url: &str) -> bool {
    let t = url.trim().to_ascii_lowercase();
    t.starts_with("http://") || t.starts_with("ws://")
}

pub fn url_has_query_secret(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("sig=")
        || lower.contains("access_token=")
        || lower.contains("api_key=")
        || lower.contains("pat=")
        || url.contains('@')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_urls() {
        assert!(assert_local_codex(Path::new("https://example/codex")).is_err());
        assert!(assert_local_codex(Path::new(r"\\share\codex.exe")).is_err());
        assert!(assert_local_codex(Path::new("/usr/bin/curl")).is_err());
    }

    #[test]
    fn cleartext_and_tokens() {
        assert!(is_cleartext_url("http://aoai.example/openai/v1"));
        assert!(!is_cleartext_url("https://aoai.example/openai/v1"));
        assert!(url_has_query_secret("https://x?token=abc"));
    }
}
