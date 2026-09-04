use crate::local_env::{env_lookup, load_merged_env, redact_url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupIssue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub ready: bool,
    pub host: String,
    pub codex_found: bool,
    pub codex_path: Option<String>,
    pub codex_version: Option<String>,
    pub codex_home: String,
    pub config_toml_exists: bool,
    pub auth_json_exists: bool,
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub azure_endpoint: Option<String>,
    pub env_key_name: Option<String>,
    pub env_key_present: bool,
    pub suggested_workspace: Option<String>,
    pub issues: Vec<SetupIssue>,
}

#[derive(Debug, Clone)]
pub struct ExecOpts {
    pub workdir: PathBuf,
    pub sandbox: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEvent {
    pub kind: String,
    pub text: String,
    pub thread_id: Option<String>,
}

pub fn default_codex_home() -> PathBuf {
    if let Ok(home) = std::env::var("CODEX_HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    if let Some(user) = dirs_home() {
        return user.join(".codex");
    }
    PathBuf::from(".codex")
}

fn dirs_home() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("USERPROFILE") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    None
}

fn extra_codex_search_paths() -> Vec<PathBuf> {
    let mut extras = Vec::new();
    if let Some(home) = dirs_home() {
        extras.push(home.join(".local").join("bin"));
        extras.push(home.join("AppData").join("Roaming").join("npm"));
        extras.push(home.join("AppData").join("Local").join("fnm_multishells"));
    }
    extras.push(PathBuf::from(r"C:\Program Files\nodejs"));
    extras
}

pub fn find_codex() -> Option<PathBuf> {
    let names = ["codex", "codex.exe", "codex.cmd"];
    for name in names {
        if let Ok(path) = which::which(name) {
            return Some(path);
        }
    }
    for dir in extra_codex_search_paths() {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn run_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Some(stdout.lines().next().unwrap_or(&stdout).to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        None
    } else {
        Some(stderr.lines().next().unwrap_or(&stderr).to_string())
    }
}

fn parse_codex_config(path: &Path) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return (None, None, None, None);
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return (None, None, None, None);
    };
    let model = value.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
    let provider = value
        .get("model_provider")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let mut endpoint = None;
    let mut env_key = None;
    if let Some(providers) = value.get("model_providers").and_then(|v| v.as_table()) {
        let chosen = provider
            .as_deref()
            .and_then(|name| providers.get(name))
            .or_else(|| providers.get("azure"));
        if let Some(block) = chosen.and_then(|v| v.as_table()) {
            endpoint = block
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(|s| redact_url(s));
            env_key = block
                .get("env_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }
    (model, provider, endpoint, env_key)
}

pub fn probe_status(app_data: Option<&Path>, cwd: &Path, host: &str) -> RuntimeStatus {
    let local = load_merged_env(app_data, cwd);
    let binary = find_codex();
    let version = binary.as_ref().and_then(|p| run_version(p));
    let home = default_codex_home();
    let config_path = home.join("config.toml");
    let auth_path = home.join("auth.json");
    let (model, provider, mut endpoint, mut env_key) = if config_path.is_file() {
        parse_codex_config(&config_path)
    } else {
        (None, None, None, None)
    };

    if endpoint.is_none() {
        if let Some(from_env) = env_lookup(&local, "AZURE_LLM_ENDPOINT") {
            endpoint = Some(redact_url(&from_env));
        }
    }
    if env_key.is_none() {
        if env_lookup(&local, "AZURE_LLM_PAT").is_some() {
            env_key = Some("AZURE_LLM_PAT".into());
        } else if env_lookup(&local, "AZURE_OPENAI_API_KEY").is_some() {
            env_key = Some("AZURE_OPENAI_API_KEY".into());
        }
    }

    let env_key_present = match env_key.as_deref() {
        Some(name) => env_lookup(&local, name).is_some(),
        None => {
            env_lookup(&local, "AZURE_LLM_PAT").is_some()
                || env_lookup(&local, "AZURE_OPENAI_API_KEY").is_some()
        }
    };

    let mut issues = Vec::new();
    if binary.is_none() {
        issues.push(SetupIssue {
            code: "codex_missing".into(),
            message: "The `codex` CLI was not found on PATH. Install OpenAI Codex, then restart Codex Desk.".into(),
        });
    }
    if !config_path.is_file() && !auth_path.is_file() && !env_key_present {
        issues.push(SetupIssue {
            code: "codex_unconfigured".into(),
            message: format!(
                "No Codex config at {}. Add config.toml (Azure endpoint) and set AZURE_LLM_PAT in the environment or .env.local.",
                home.display()
            ),
        });
    }
    if provider.as_deref() == Some("azure") && !env_key_present {
        issues.push(SetupIssue {
            code: "azure_pat_missing".into(),
            message: format!(
                "Codex is set to Azure, but {} is not set. Put the PAT in that environment variable — not in the repo.",
                env_key.clone().unwrap_or_else(|| "AZURE_LLM_PAT".into())
            ),
        });
    }
    if endpoint.is_none() && provider.as_deref() == Some("azure") {
        issues.push(SetupIssue {
            code: "azure_endpoint_missing".into(),
            message: "Azure provider is selected, but no endpoint was found in Codex config.toml or AZURE_LLM_ENDPOINT.".into(),
        });
    }

    RuntimeStatus {
        ready: binary.is_some(),
        host: host.to_string(),
        codex_found: binary.is_some(),
        codex_path: binary.as_ref().map(|p| p.display().to_string()),
        codex_version: version,
        codex_home: home.display().to_string(),
        config_toml_exists: config_path.is_file(),
        auth_json_exists: auth_path.is_file(),
        model,
        model_provider: provider,
        azure_endpoint: endpoint,
        env_key_name: env_key,
        env_key_present,
        suggested_workspace: Some(cwd.display().to_string()),
        issues,
    }
}

pub fn apply_codex_env(cmd: &mut Command, app_data: Option<&Path>, cwd: &Path) {
    let local = load_merged_env(app_data, cwd);
    for (key, value) in &local {
        if std::env::var(key).ok().filter(|v| !v.is_empty()).is_none() {
            cmd.env(key, value);
        }
    }
    let pat = env_lookup(&local, "AZURE_LLM_PAT");
    let openai = env_lookup(&local, "AZURE_OPENAI_API_KEY");
    if openai.is_none() {
        if let Some(pat) = pat {
            cmd.env("AZURE_OPENAI_API_KEY", pat);
        }
    }
}

pub fn validate_workspace(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Set a workspace path on the agent. Desk will not write into your home directory by default.".into());
    }
    let candidate = PathBuf::from(trimmed);
    let canon = candidate.canonicalize().map_err(|_| {
        format!("Workspace does not exist: {trimmed}")
    })?;
    if !canon.is_dir() {
        return Err("Workspace must be a directory.".into());
    }
    if let Some(home) = dirs_home() {
        if let Ok(home_canon) = home.canonicalize() {
            if canon == home_canon {
                return Err("Refusing the user home directory as a workspace. Point the agent at a specific repo checkout.".into());
            }
        }
    }
    if canon == PathBuf::from("/") || canon == PathBuf::from(r"C:\") {
        return Err("Refusing filesystem root as a workspace.".into());
    }
    Ok(canon)
}

pub fn workspace_dir(app_data: &Path) -> Result<PathBuf, String> {
    let dir = app_data.join("workspace");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create workspace: {e}"))?;
    Ok(dir)
}

fn apply_windows_flags(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

pub fn run_turn(
    binary: &Path,
    thread_id: Option<&str>,
    prompt: &str,
    app_data: &Path,
    project_cwd: &Path,
    cancel: Arc<AtomicBool>,
    opts: Option<&ExecOpts>,
    mut on_event: impl FnMut(CodexEvent),
) -> Result<(String, Option<String>), String> {
    let default_dir = workspace_dir(app_data)?;
    let workdir = opts
        .map(|o| o.workdir.clone())
        .unwrap_or(default_dir);
    let sandbox = opts
        .map(|o| o.sandbox.as_str())
        .unwrap_or("read-only");
    let mut cmd = Command::new(binary);
    cmd.arg("exec");
    if let Some(id) = thread_id {
        cmd.arg("resume").arg(id);
    }
    cmd.args([
        "--json",
        "--skip-git-repo-check",
        "--sandbox",
        sandbox,
        "--ask-for-approval",
        "never",
        "-",
    ]);
    cmd.current_dir(&workdir);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    apply_codex_env(&mut cmd, Some(app_data), project_cwd);
    apply_windows_flags(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to start Codex CLI at {}: {e}", binary.display())
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("write prompt to Codex: {e}"))?;
    }

    let stdout = child.stdout.take().ok_or_else(|| "Codex stdout missing".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "Codex stderr missing".to_string())?;

    let cancel_err = cancel.clone();
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for line in reader.lines().flatten() {
            if cancel_err.load(Ordering::Relaxed) {
                break;
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !looks_like_secret_line(trimmed) {
                lines.push(trimmed.to_string());
            }
        }
        lines
    });

    let mut assistant = String::new();
    let mut seen_thread: Option<String> = None;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            return Err("Cancelled.".into());
        }
        let Ok(line) = line else {
            continue;
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if let Some(event) = map_json_event(&value) {
                if let Some(id) = event.thread_id.clone() {
                    seen_thread = Some(id);
                }
                if event.kind == "assistant" && !event.text.is_empty() {
                    assistant = event.text.clone();
                }
                on_event(event);
            }
        } else if !looks_like_secret_line(trimmed) {
            if !assistant.is_empty() {
                assistant.push('\n');
            }
            assistant.push_str(trimmed);
            on_event(CodexEvent {
                kind: "assistant".into(),
                text: assistant.clone(),
                thread_id: seen_thread.clone(),
            });
        }
    }

    let status = child.wait().map_err(|e| format!("wait for Codex: {e}"))?;
    let stderr_lines = stderr_thread.join().unwrap_or_default();
    let stderr_text = stderr_lines.join("\n");

    if !status.success() {
        let detail = if !stderr_text.is_empty() {
            redact_process_output(&stderr_text)
        } else if !assistant.is_empty() {
            redact_process_output(&assistant)
        } else {
            format!("Codex exited with status {status}.")
        };
        return Err(explain_codex_failure(&detail));
    }

    if assistant.is_empty() {
        if !stderr_text.is_empty() {
            assistant = redact_process_output(&stderr_text);
        }
    }

    if assistant.is_empty() {
        assistant = "(Codex finished without a visible reply.)".into();
    }

    on_event(CodexEvent {
        kind: "assistant".into(),
        text: assistant.clone(),
        thread_id: seen_thread.clone(),
    });

    Ok((assistant, seen_thread))
}

fn looks_like_secret_line(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    SECRETISH.iter().any(|m| upper.contains(m)) && (line.contains('=') || line.contains(':'))
}

const SECRETISH: &[&str] = &["PAT", "API_KEY", "TOKEN", "SECRET", "BEARER "];

fn redact_process_output(text: &str) -> String {
    text.lines()
        .map(|line| {
            if looks_like_secret_line(line) {
                "[redacted line]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn explain_codex_failure(detail: &str) -> String {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("401") || lower.contains("unauthorized") || lower.contains("forbidden") {
        return format!(
            "Codex could not authenticate to the Azure-hosted model. Check AZURE_LLM_PAT (or the env_key in {home}/config.toml) and that the token is valid.\n\n{detail}",
            home = default_codex_home().display()
        );
    }
    if lower.contains("enotfound") || lower.contains("dns") || lower.contains("getaddrinfo") {
        return format!(
            "Codex could not reach the Azure endpoint. Confirm AZURE_LLM_ENDPOINT / config.toml base_url (no credentials in the URL).\n\n{detail}"
        );
    }
    if lower.contains("login") || lower.contains("auth") {
        return format!(
            "Codex is installed but not ready. Finish Codex setup (config.toml + PAT env var, or `codex login` if you use that flow).\n\n{detail}"
        );
    }
    format!("Codex reported an error:\n\n{detail}")
}

fn map_json_event(value: &Value) -> Option<CodexEvent> {
    let typ = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if typ == "thread.started" {
        let id = value
            .get("thread_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        return Some(CodexEvent {
            kind: "thread".into(),
            text: String::new(),
            thread_id: id,
        });
    }
    if typ == "turn.started" {
        return Some(CodexEvent {
            kind: "status".into(),
            text: "Codex started a turn…".into(),
            thread_id: None,
        });
    }
    if typ == "turn.failed" || typ == "error" {
        let text = value
            .get("error")
            .and_then(|v| v.as_str())
            .or_else(|| value.get("message").and_then(|v| v.as_str()))
            .unwrap_or("Codex turn failed.")
            .to_string();
        return Some(CodexEvent {
            kind: "error".into(),
            text,
            thread_id: None,
        });
    }
    if typ == "item.started" || typ == "item.updated" || typ == "item.completed" {
        if let Some(item) = value.get("item") {
            let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if item_type == "agent_message" {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    return Some(CodexEvent {
                        kind: "assistant".into(),
                        text: text.to_string(),
                        thread_id: None,
                    });
                }
            }
            if item_type == "reasoning" {
                let text = item
                    .get("text")
                    .or_else(|| item.get("summary"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Thinking…");
                return Some(CodexEvent {
                    kind: "status".into(),
                    text: text.to_string(),
                    thread_id: None,
                });
            }
            if item_type == "command_execution" {
                let cmd = item.get("command").and_then(|v| v.as_str()).unwrap_or("command");
                return Some(CodexEvent {
                    kind: "status".into(),
                    text: format!("Codex ran: {cmd}"),
                    thread_id: None,
                });
            }
        }
    }
    if let Some(text) = value.pointer("/item/text").and_then(|v| v.as_str()) {
        if !text.is_empty() && typ.contains("message") {
            return Some(CodexEvent {
                kind: "assistant".into(),
                text: text.to_string(),
                thread_id: None,
            });
        }
    }
    None
}
