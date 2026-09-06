//! Setup / Env inventory from Codex `config.toml`.
//!
//! Reads `CODEX_HOME` or `~/.codex` / `%USERPROFILE%\.codex`.
//! Lists every `env_key` plus Azure template vars. Values never returned
//! for secrets — only FOUND / MISSING and a plain-English description.

use crate::codex::default_codex_home;
use crate::env_vault;
use crate::local_env::{env_lookup, load_merged_env, redact_url};
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct EnvVarRow {
    pub key: String,
    pub kind: String,
    pub description: String,
    pub status: String,
    pub source: String,
    pub required: bool,
    pub from_config: bool,
    pub related_to: Option<String>,
    pub display_value: Option<String>,
    pub settable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigFieldRow {
    pub key: String,
    pub description: String,
    pub status: String,
    pub display_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetupEnvStatus {
    pub codex_home: String,
    pub config_path: String,
    pub config_toml_exists: bool,
    pub home_source: String,
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub base_url: Option<String>,
    pub env_keys_in_config: Vec<String>,
    pub vars: Vec<EnvVarRow>,
    pub config_fields: Vec<ConfigFieldRow>,
    pub note: String,
}

pub fn describe_env_key(key: &str) -> &'static str {
    match key {
        "AZURE_OPENAI_API_KEY" => {
            "API key / PAT the Codex Azure provider reads. Point config.toml env_key here, or let Desk export AZURE_LLM_PAT as this name to the child Codex process only."
        }
        "AZURE_LLM_PAT" => {
            "Desk-preferred name for the Azure PAT. Never put this in config.toml. Desk exports it to child Codex as AZURE_OPENAI_API_KEY when that var is unset."
        }
        "AZURE_LLM_ENDPOINT" => {
            "Optional HTTPS Azure base URL when config.toml has no base_url. Desk does not open Azure sockets."
        }
        "AZURE_OPENAI_ENDPOINT" => {
            "Standard Azure OpenAI endpoint name. Related documentation alias — Desk is not an Azure SDK client."
        }
        "AZURE_OPENAI_DEPLOYMENT" => {
            "Azure deployment / model name some tools expect. Prefer config.toml model= for Codex."
        }
        "OPENAI_API_KEY" => {
            "Generic Codex env_key some configs use. If config.toml points here, set this in the Desk vault or the process environment."
        }
        "CODEX_API_KEY" => {
            "Alternate Codex API key name. Only needed if your config.toml env_key uses it."
        }
        other if other.contains("KEY") || other.contains("PAT") || other.contains("TOKEN") => {
            "Secret referenced by Codex config.toml env_key. Store in the Desk vault or the process environment — never in config.toml."
        }
        _ => "Environment variable referenced by Codex config or the Azure template. Desk exports vault values only to the child Codex process.",
    }
}

fn describe_config_field(key: &str) -> &'static str {
    match key {
        "model" => "Azure deployment / model name in config.toml. Codex Desk will not invent one.",
        "model_provider" => "Selected Codex provider (azure). Desk does not invent a second client.",
        "base_url" => "HTTPS Azure resource endpoint in config.toml. No PAT in the URL.",
        "env_key" => "Name of the environment variable that holds the PAT. The value stays out of config.toml.",
        _ => "Codex config.toml field.",
    }
}

fn collect_env_keys(value: &toml::Value, out: &mut BTreeSet<String>) {
    match value {
        toml::Value::Table(table) => {
            if let Some(key) = table.get("env_key").and_then(|v| v.as_str()) {
                if !key.trim().is_empty() {
                    out.insert(key.trim().to_string());
                }
            }
            for v in table.values() {
                collect_env_keys(v, out);
            }
        }
        toml::Value::Array(items) => {
            for v in items {
                collect_env_keys(v, out);
            }
        }
        _ => {}
    }
}

fn home_source() -> &'static str {
    if std::env::var("CODEX_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        "CODEX_HOME"
    } else if std::env::var("USERPROFILE")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        "USERPROFILE"
    } else if std::env::var("HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        "HOME"
    } else {
        "fallback"
    }
}

fn lookup_source(
    app_data: Option<&Path>,
    cwd: &Path,
    key: &str,
) -> (bool, &'static str) {
    let local = load_merged_env(app_data, cwd);
    if env_lookup(&local, key).is_some() {
        if local.get(key).map(|v| !v.is_empty()).unwrap_or(false) {
            return (true, "env-file");
        }
        return (true, "process");
    }
    if let Some(dir) = app_data {
        if env_vault::has_key(dir, key) {
            return (true, "desk-vault");
        }
        if key == "AZURE_LLM_PAT" && crate::keystore::pat_slot_present(dir) {
            return (true, "os-slot");
        }
    }
    (false, "missing")
}

fn related_for(key: &str, config_keys: &BTreeSet<String>) -> Option<String> {
    if key == "AZURE_OPENAI_API_KEY" && config_keys.contains("AZURE_LLM_PAT") {
        return Some("AZURE_LLM_PAT".into());
    }
    if key == "AZURE_LLM_PAT" && config_keys.contains("AZURE_OPENAI_API_KEY") {
        return Some("AZURE_OPENAI_API_KEY".into());
    }
    if key == "AZURE_LLM_ENDPOINT" {
        return Some("base_url".into());
    }
    if key == "AZURE_OPENAI_ENDPOINT" {
        return Some("base_url".into());
    }
    if key == "AZURE_OPENAI_DEPLOYMENT" {
        return Some("model".into());
    }
    None
}

pub fn setup_env_status(app_data: Option<&Path>, cwd: &Path) -> SetupEnvStatus {
    let home = default_codex_home();
    let config_path = home.join("config.toml");
    let exists = config_path.is_file();
    let mut env_keys = BTreeSet::new();
    let mut model = None;
    let mut provider = None;
    let mut base_url = None;
    if exists {
        if let Ok(text) = std::fs::read_to_string(&config_path) {
            if let Ok(value) = text.parse::<toml::Value>() {
                model = value.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
                provider = value
                    .get("model_provider")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                collect_env_keys(&value, &mut env_keys);
                if let Some(providers) = value.get("model_providers").and_then(|v| v.as_table()) {
                    let chosen = provider
                        .as_deref()
                        .and_then(|name| providers.get(name))
                        .or_else(|| providers.get("azure"));
                    if let Some(block) = chosen.and_then(|v| v.as_table()) {
                        base_url = block
                            .get("base_url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
            }
        }
    }

    let mut wanted: BTreeSet<String> = env_keys.clone();
    for standard in [
        "AZURE_OPENAI_API_KEY",
        "AZURE_LLM_PAT",
        "AZURE_LLM_ENDPOINT",
        "AZURE_OPENAI_ENDPOINT",
        "AZURE_OPENAI_DEPLOYMENT",
    ] {
        wanted.insert(standard.into());
    }
    if let Some(dir) = app_data {
        for key in env_vault::present_keys(dir) {
            wanted.insert(key);
        }
    }

    let mut vars = Vec::new();
    for key in wanted {
        let (found, source) = lookup_source(app_data, cwd, &key);
        let from_config = env_keys.contains(&key);
        let required = from_config
            || key == "AZURE_OPENAI_API_KEY"
            || key == "AZURE_LLM_PAT";
        vars.push(EnvVarRow {
            related_to: related_for(&key, &env_keys),
            description: describe_env_key(&key).to_string(),
            status: if found { "FOUND".into() } else { "MISSING".into() },
            source: source.into(),
            required,
            from_config,
            display_value: None,
            settable: true,
            kind: "env".into(),
            key,
        });
    }
    vars.sort_by(|a, b| {
        b.from_config
            .cmp(&a.from_config)
            .then(b.required.cmp(&a.required))
            .then(a.key.cmp(&b.key))
    });

    let redacted_url = base_url.as_deref().map(redact_url);
    let config_fields = vec![
        ConfigFieldRow {
            key: "model".into(),
            description: describe_config_field("model").into(),
            status: if model.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                "FOUND".into()
            } else {
                "MISSING".into()
            },
            display_value: model.clone(),
        },
        ConfigFieldRow {
            key: "base_url".into(),
            description: describe_config_field("base_url").into(),
            status: if redacted_url.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                "FOUND".into()
            } else {
                "MISSING".into()
            },
            display_value: redacted_url.clone(),
        },
        ConfigFieldRow {
            key: "model_provider".into(),
            description: describe_config_field("model_provider").into(),
            status: if provider.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                "FOUND".into()
            } else {
                "MISSING".into()
            },
            display_value: provider.clone(),
        },
        ConfigFieldRow {
            key: "env_key".into(),
            description: describe_config_field("env_key").into(),
            status: if env_keys.is_empty() {
                "MISSING".into()
            } else {
                "FOUND".into()
            },
            display_value: if env_keys.is_empty() {
                None
            } else {
                Some(env_keys.iter().cloned().collect::<Vec<_>>().join(", "))
            },
        },
    ];

    SetupEnvStatus {
        codex_home: home.display().to_string(),
        config_path: config_path.display().to_string(),
        config_toml_exists: exists,
        home_source: home_source().into(),
        model,
        model_provider: provider,
        base_url: redacted_url,
        env_keys_in_config: env_keys.into_iter().collect(),
        vars,
        config_fields,
        note: "Desk reads Codex config.toml only. Vault values export to the child codex process — Desk is not an Azure SDK.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn lists_every_env_key_and_azure_template() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            home.join("config.toml"),
            r#"
model = "desk-deploy"
model_provider = "azure"

[model_providers.azure]
base_url = "https://example.openai.azure.com/openai/v1"
env_key = "AZURE_LLM_PAT"

[model_providers.other]
env_key = "OPENAI_API_KEY"
"#,
        )
        .unwrap();
        let prev = std::env::var("CODEX_HOME").ok();
        std::env::set_var("CODEX_HOME", &home);
        let status = setup_env_status(Some(dir.path()), dir.path());
        match prev {
            Some(v) => std::env::set_var("CODEX_HOME", v),
            None => std::env::remove_var("CODEX_HOME"),
        }
        let keys: Vec<_> = status.vars.iter().map(|v| v.key.as_str()).collect();
        assert!(keys.contains(&"AZURE_LLM_PAT"));
        assert!(keys.contains(&"OPENAI_API_KEY"));
        assert!(keys.contains(&"AZURE_OPENAI_API_KEY"));
        assert!(keys.contains(&"AZURE_LLM_ENDPOINT"));
        assert!(status.vars.iter().any(|v| v.key == "AZURE_LLM_PAT" && v.status == "MISSING"));
        assert_eq!(status.model.as_deref(), Some("desk-deploy"));
        assert!(status.config_toml_exists);
        assert!(status.note.contains("not an Azure SDK"));
        assert!(describe_env_key("AZURE_OPENAI_API_KEY").contains("PAT"));
    }

    #[test]
    fn vault_marks_found_without_exposing_value() {
        let dir = TempDir::new().unwrap();
        crate::env_vault::set_value(dir.path(), "AZURE_LLM_PAT", "placeholder-not-a-real-pat").unwrap();
        let home = dir.path().join("empty-home");
        std::fs::create_dir_all(&home).unwrap();
        let prev = std::env::var("CODEX_HOME").ok();
        std::env::set_var("CODEX_HOME", &home);
        let status = setup_env_status(Some(dir.path()), dir.path());
        match prev {
            Some(v) => std::env::set_var("CODEX_HOME", v),
            None => std::env::remove_var("CODEX_HOME"),
        }
        let pat = status.vars.iter().find(|v| v.key == "AZURE_LLM_PAT").unwrap();
        assert_eq!(pat.status, "FOUND");
        assert_eq!(pat.source, "desk-vault");
        assert!(pat.display_value.is_none());
    }
}
