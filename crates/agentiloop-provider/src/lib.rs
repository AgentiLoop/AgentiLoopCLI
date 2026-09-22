//! Model backends: Anthropic Messages API, OpenAI-compatible Chat Completions
//! (OpenAI, Ollama, LM Studio, Groq, OpenRouter, …), and oMLX (local MLX server).

pub mod anthropic;
pub mod omlx;
pub mod openai;

use std::sync::Arc;

use agentiloop_core::Provider;
pub use agentiloop_core::ModelInfo;
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;

/// Build a provider by name (`anthropic` | `openai` | `omlx`), or pick one from
/// the environment when `name` is `None`: Anthropic if `ANTHROPIC_API_KEY` /
/// `ANTHROPIC_OAUTH_TOKEN` is set, otherwise OpenAI if `OPENAI_API_KEY` or
/// `OPENAI_BASE_URL` is set, otherwise oMLX if `OMLX_BASE_URL` / `OMLX_PORT` /
/// `OMLX_API_KEY` is set.
pub fn from_env(name: Option<&str>) -> anyhow::Result<Arc<dyn Provider>> {
    let has = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());
    let name = match name {
        Some(n) => n.to_ascii_lowercase(),
        None if has("ANTHROPIC_API_KEY") || has("ANTHROPIC_OAUTH_TOKEN") => "anthropic".into(),
        None if has("OPENAI_API_KEY") || has("OPENAI_BASE_URL") => "openai".into(),
        None if has("OMLX_BASE_URL") || has("OMLX_PORT") || has("OMLX_API_KEY") => "omlx".into(),
        None => anyhow::bail!(
            "no provider credentials found: set ANTHROPIC_API_KEY, OPENAI_API_KEY / OPENAI_BASE_URL (e.g. http://localhost:11434/v1 for Ollama), or use `-p omlx` for a local oMLX server"
        ),
    };
    Ok(match name.as_str() {
        "anthropic" => Arc::new(AnthropicProvider::from_env()?),
        "openai" => Arc::new(OpenAIProvider::from_env()?),
        "omlx" => Arc::new(omlx::from_env()?),
        other => anyhow::bail!("unknown provider `{other}` (expected anthropic, openai, or omlx)"),
    })
}
