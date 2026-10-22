//! agentiloop-core: provider-agnostic message model, tool trait, and the agentic loop.

pub mod message;
pub mod tool;
pub mod provider;
pub mod agent;
pub mod permission;
pub mod session;
pub mod instructions;
pub mod commands;
pub mod hooks;
pub mod mentions;
pub mod subagent;

pub use agent::{is_transient, retry_delay, Agent, AgentConfig, AgentEvent, ModelLimits, Usage, DEFAULT_MAX_RETRIES};
pub use hooks::Hooks;
pub use subagent::TaskTool;
pub use message::{ContentBlock, Message, Role, StopReason};
pub use permission::{Permission, PermissionPolicy};
pub use provider::{ModelInfo, Provider, ProviderRequest, ProviderResponse, ToolSpec};
pub use session::Session;
pub use tool::{Tool, ToolContext, ToolError, ToolRegistry, ToolResult};
