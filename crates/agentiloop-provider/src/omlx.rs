//! oMLX (<https://omlx.ai>) — macOS-native MLX inference server with paged SSD
//! KV caching and continuous batching. It exposes OpenAI-compatible Chat
//! Completions on `http://localhost:8000/v1`, so this is the OpenAI backend
//! under its own name with oMLX defaults.

use crate::OpenAIProvider;

/// Base URL honouring oMLX's own `OMLX_PORT` setting.
fn default_base_url() -> String {
    let port = std::env::var("OMLX_PORT").ok().filter(|p| !p.is_empty()).unwrap_or_else(|| "8000".into());
    format!("http://localhost:{port}/v1")
}

/// Reads `OMLX_BASE_URL` (default `http://localhost:8000/v1`, or `OMLX_PORT`)
/// and `OMLX_API_KEY` (only needed when the server was started with one).
/// No default model: oMLX serves whatever is in its model directory, so the
/// first entry of `/v1/models` is used unless `--model` is given.
pub fn from_env() -> anyhow::Result<OpenAIProvider> {
    let base_url = std::env::var("OMLX_BASE_URL").ok().filter(|u| !u.is_empty()).unwrap_or_else(default_base_url);
    let key = std::env::var("OMLX_API_KEY").unwrap_or_else(|_| "none".into());
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
}
