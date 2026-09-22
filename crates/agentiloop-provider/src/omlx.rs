//! oMLX (<https://omlx.ai>) — macOS-native MLX inference server with paged SSD
//! KV caching and continuous batching. It exposes OpenAI-compatible Chat
//! Completions on `http://localhost:<port>/v1`, so this is the OpenAI backend
//! under its own name with oMLX defaults.

use std::path::PathBuf;

use crate::OpenAIProvider;

/// `server.port` and `auth.api_key` from oMLX's own settings file, so a
/// locally installed server works with no env vars at all.
struct Settings {
    port: Option<u16>,
    api_key: Option<String>,
}

fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".omlx").join("settings.json"))
}

fn read_settings(path: Option<PathBuf>) -> Settings {
    let empty = Settings { port: None, api_key: None };
    let Some(text) = path.and_then(|p| std::fs::read_to_string(p).ok()) else { return empty };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return empty };
    let auth = &v["auth"];
    // A key is only required when verification is on.
    let api_key = if auth["skip_api_key_verification"].as_bool().unwrap_or(false) {
        None
    } else {
        auth["api_key"].as_str().filter(|k| !k.is_empty()).map(str::to_string)
    };
    Settings { port: v["server"]["port"].as_u64().and_then(|p| u16::try_from(p).ok()), api_key }
}

/// Base URL: `OMLX_PORT`, else the port in `~/.omlx/settings.json`, else 8000.
fn default_base_url(settings: &Settings) -> String {
    let port = std::env::var("OMLX_PORT")
        .ok()
        .filter(|p| !p.is_empty())
        .or_else(|| settings.port.map(|p| p.to_string()))
        .unwrap_or_else(|| "8000".into());
    format!("http://localhost:{port}/v1")
}

/// Reads `OMLX_BASE_URL` (or `OMLX_PORT`) and `OMLX_API_KEY`; anything not set
/// falls back to `~/.omlx/settings.json`. No default model: oMLX serves
/// whatever is in its model directory, so the first entry of `/v1/models` is
/// used unless `--model` is given.
pub fn from_env() -> anyhow::Result<OpenAIProvider> {
    let settings = read_settings(settings_path());
    let base_url = std::env::var("OMLX_BASE_URL").ok().filter(|u| !u.is_empty()).unwrap_or_else(|| default_base_url(&settings));
    let key = std::env::var("OMLX_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .or(settings.api_key)
        .unwrap_or_else(|| "none".into());
    Ok(OpenAIProvider::new(key, base_url).with_identity("omlx", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentiloop_core::Provider;

    #[test]
    fn identity_is_omlx_with_no_fixed_default_model() {
        let p = from_env().unwrap();
        assert_eq!(p.name(), "omlx");
        assert_eq!(p.default_model(), "");
    }

    #[test]
    fn settings_file_supplies_port_and_key() {
        let dir = std::env::temp_dir().join(format!("omlx-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("settings.json");
        std::fs::write(&p, r#"{"server":{"port":7777},"auth":{"api_key":"sk-omlx-abc","skip_api_key_verification":false}}"#).unwrap();
        let s = read_settings(Some(p.clone()));
        assert_eq!(s.port, Some(7777));
        assert_eq!(s.api_key.as_deref(), Some("sk-omlx-abc"));
        assert_eq!(default_base_url(&s), "http://localhost:7777/v1");
        // Verification off → no key needed.
        std::fs::write(&p, r#"{"server":{"port":7777},"auth":{"api_key":"sk","skip_api_key_verification":true}}"#).unwrap();
        assert!(read_settings(Some(p)).api_key.is_none());
        // Missing/garbage file → defaults.
        let none = read_settings(Some(dir.join("nope.json")));
        assert!(none.port.is_none() && none.api_key.is_none());
        assert_eq!(default_base_url(&none), "http://localhost:8000/v1");
        std::fs::remove_dir_all(dir).ok();
    }
}
