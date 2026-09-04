use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SECRET_MARKERS: &[&str] = &["PAT", "KEY", "TOKEN", "SECRET", "PASSWORD", "PASS"];

#[allow(dead_code)]
pub fn is_secret_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    SECRET_MARKERS.iter().any(|marker| upper.contains(marker))
}

pub fn parse_env_file(path: &Path) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = value.trim().to_string();
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value = value[1..value.len() - 1].to_string();
        }
        out.insert(key.to_string(), value);
    }
    out
}

pub fn candidate_env_files(app_data: Option<&Path>, cwd: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(explicit) = std::env::var("CODEX_DESK_ENV_FILE") {
        if !explicit.is_empty() {
            paths.push(PathBuf::from(explicit));
        }
    }
    if let Some(dir) = app_data {
        paths.push(dir.join(".env.local"));
        paths.push(dir.join(".env"));
    }
    paths.push(cwd.join(".env.local"));
    paths.push(cwd.join(".env"));
    paths
}

pub fn load_merged_env(app_data: Option<&Path>, cwd: &Path) -> HashMap<String, String> {
    let mut merged = HashMap::new();
    for path in candidate_env_files(app_data, cwd) {
        if path.is_file() {
            for (k, v) in parse_env_file(&path) {
                merged.entry(k).or_insert(v);
            }
        }
    }
    merged
}

pub fn env_lookup(local: &HashMap<String, String>, key: &str) -> Option<String> {
    if let Some(value) = local.get(key) {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

pub fn redact_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.contains('@') || trimmed.contains("token=") || trimmed.contains("sig=") {
        return "(redacted — remove credentials from the URL; use AZURE_LLM_PAT instead)".into();
    }
    trimmed.to_string()
}
