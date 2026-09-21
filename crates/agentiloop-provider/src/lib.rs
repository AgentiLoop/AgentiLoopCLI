//! Model backends. Currently: Anthropic Messages API. OpenAI-compatible next.

pub mod anthropic;

pub use anthropic::AnthropicProvider;
