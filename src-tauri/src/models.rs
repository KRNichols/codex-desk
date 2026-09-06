//! Workspace models catalog — slugs only.
//!
//! `config/models.json` is a rewriteable slug list. It must not contain
//! system/developer prompts, secrets, or provider endpoints. Desk injects
//! `briefs/OPERATOR.md` via exec `--config developer_instructions`, not
//! via this file. Provider + `base_url` + `env_key` stay in Codex
//! `config.toml`.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

const CATALOG_REL: &str = "config/models.json";

const CATALOG_NOTE: &str = "Slug catalog only. No secrets and no system/developer prompts. Desk injects briefs/OPERATOR.md via exec --config. Set config.toml model= to a catalog slug.";

/// Keys that must never appear in the models catalog (any nesting).
/// Compare after lowercasing and stripping `_` / `-`.
const FORBIDDEN_KEYS: &[&str] = &[
    "system",
    "systemprompt",
    "systeminstructions",
    "developerinstructions",
    "developerprompt",
    "modelinstructions",
    "modelinstructionsfile",
    "instructions",
    "prompt",
    "prompts",
    "messages",
    "apikey",
    "pat",
    "token",
    "secret",
    "bearer",
    "baseurl",
    "endpoint",
    "envkey",
    "wireapi",
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogModel {
    pub slug: String,
    pub label: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelsCatalog {
    pub path: String,
    pub exists: bool,
    pub ok: bool,
    pub error: Option<String>,
    pub slugs: Vec<String>,
    pub models: Vec<CatalogModel>,
    pub note: String,
}

pub fn catalog_path(cwd: &Path) -> PathBuf {
    cwd.join("config").join("models.json")
}

pub fn empty_catalog(cwd: &Path) -> ModelsCatalog {
    ModelsCatalog {
        path: catalog_path(cwd).display().to_string(),
        exists: false,
        ok: false,
        error: Some(format!("No {CATALOG_REL} in the workspace. Add a slug catalog (no prompts, no secrets).")),
        slugs: Vec::new(),
        models: Vec::new(),
        note: CATALOG_NOTE.into(),
    }
}

pub fn load_catalog(cwd: &Path) -> ModelsCatalog {
    let path = catalog_path(cwd);
    if !path.is_file() {
        return empty_catalog(cwd);
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(err) => {
            return ModelsCatalog {
                path: path.display().to_string(),
                exists: true,
                ok: false,
                error: Some(format!("Could not read {CATALOG_REL}: {err}")),
                slugs: Vec::new(),
                models: Vec::new(),
                note: CATALOG_NOTE.into(),
            };
        }
    };
    match parse_catalog(&text) {
        Ok(models) => ModelsCatalog {
            path: path.display().to_string(),
            exists: true,
            ok: true,
            error: None,
            slugs: models.iter().map(|m| m.slug.clone()).collect(),
            models,
            note: CATALOG_NOTE.into(),
        },
        Err(error) => ModelsCatalog {
            path: path.display().to_string(),
            exists: true,
            ok: false,
            error: Some(error),
            slugs: Vec::new(),
            models: Vec::new(),
            note: CATALOG_NOTE.into(),
        },
    }
}

pub fn parse_catalog(text: &str) -> Result<Vec<CatalogModel>, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("models.json is not valid JSON: {e}"))?;
    let forbidden = collect_forbidden(&value);
    if !forbidden.is_empty() {
        return Err(format!(
            "models.json must be slugs/catalog only. Remove prompt/secret/endpoint keys: {}",
            forbidden.join(", ")
        ));
    }
    let models = extract_models(&value);
    if models.is_empty() {
        return Err("models.json has no slugs. Add models[].slug (or a slugs array).".into());
    }
    Ok(models)
}

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|c| *c != '_' && *c != '-')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn collect_forbidden(value: &Value) -> Vec<String> {
    let mut found = Vec::new();
    walk_forbidden(value, &mut found);
    found.sort();
    found.dedup();
    found
}

fn walk_forbidden(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let compact = normalize_key(k);
                if FORBIDDEN_KEYS.contains(&compact.as_str()) {
                    out.push(k.clone());
                }
                walk_forbidden(v, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                walk_forbidden(v, out);
            }
        }
        _ => {}
    }
}

fn slug_from_object(obj: &serde_json::Map<String, Value>) -> Option<CatalogModel> {
    let slug = obj
        .get("slug")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("id").and_then(|v| v.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let label = obj
        .get("label")
        .or_else(|| obj.get("name"))
        .or_else(|| obj.get("display_name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let provider = obj
        .get("provider")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    Some(CatalogModel {
        slug: slug.to_string(),
        label,
        provider,
    })
}

fn extract_models(value: &Value) -> Vec<CatalogModel> {
    let mut out = Vec::new();
    match value {
        Value::Array(items) => push_entries(items, &mut out),
        Value::Object(map) => {
            if let Some(Value::Array(items)) = map.get("models").or_else(|| map.get("slugs")) {
                push_entries(items, &mut out);
            } else if let Some(Value::Object(models)) = map.get("models") {
                for (slug, entry) in models {
                    let trimmed = slug.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(obj) = entry.as_object() {
                        if let Some(mut model) = slug_from_object(obj) {
                            if model.slug != trimmed {
                                model.slug = trimmed.to_string();
                            }
                            out.push(model);
                            continue;
                        }
                    }
                    out.push(CatalogModel {
                        slug: trimmed.to_string(),
                        label: None,
                        provider: None,
                    });
                }
            }
        }
        _ => {}
    }
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|m| seen.insert(m.slug.clone()));
    out
}

fn push_entries(items: &[Value], out: &mut Vec<CatalogModel>) {
    for item in items {
        match item {
            Value::String(s) => {
                let slug = s.trim();
                if !slug.is_empty() {
                    out.push(CatalogModel {
                        slug: slug.to_string(),
                        label: None,
                        provider: None,
                    });
                }
            }
            Value::Object(obj) => {
                if let Some(model) = slug_from_object(obj) {
                    out.push(model);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = include_str!("../../config/models.json");

    #[test]
    fn shipped_catalog_is_slugs_only() {
        let models = parse_catalog(SHIPPED).expect("shipped catalog must parse");
        assert!(models.iter().any(|m| m.slug == "YOUR_AZURE_DEPLOYMENT_NAME"));
        assert!(!SHIPPED.to_ascii_lowercase().contains("system_prompt"));
        assert!(!SHIPPED.to_ascii_lowercase().contains("developer_instructions"));
        assert!(!SHIPPED.contains("sk-"));
        assert!(!SHIPPED.to_ascii_lowercase().contains("api_key"));
        assert!(!SHIPPED.contains("https://"));
    }

    #[test]
    fn rejects_system_prompt_in_catalog() {
        let err = parse_catalog(
            r#"{ "models": [{ "slug": "x", "system_prompt": "do not put this here" }] }"#,
        )
        .unwrap_err();
        assert!(err.contains("slugs/catalog only"));
        assert!(err.contains("system_prompt"));
    }

    #[test]
    fn rejects_endpoint_and_env_key() {
        let err = parse_catalog(
            r#"{ "models": [{ "slug": "x", "base_url": "https://example.invalid", "env_key": "AZURE_OPENAI_API_KEY" }] }"#,
        )
        .unwrap_err();
        assert!(err.contains("base_url") || err.contains("env_key"));
    }

    #[test]
    fn accepts_string_slugs_and_id_alias() {
        let models = parse_catalog(
            r#"{ "slugs": ["alpha", { "id": "beta", "label": "Beta" }] }"#,
        )
        .unwrap();
        assert_eq!(models[0].slug, "alpha");
        assert_eq!(models[1].slug, "beta");
        assert_eq!(models[1].label.as_deref(), Some("Beta"));
    }

    #[test]
    fn missing_file_is_not_ok() {
        let dir = tempfile::TempDir::new().unwrap();
        let catalog = load_catalog(dir.path());
        assert!(!catalog.exists);
        assert!(!catalog.ok);
        assert!(catalog.slugs.is_empty());
    }

    #[test]
    fn loads_workspace_catalog() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(
            dir.path().join("config/models.json"),
            r#"{ "models": [{ "slug": "desk-deploy", "provider": "azure" }] }"#,
        )
        .unwrap();
        let catalog = load_catalog(dir.path());
        assert!(catalog.ok);
        assert_eq!(catalog.slugs, vec!["desk-deploy".to_string()]);
    }
}
