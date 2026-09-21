//! agentiloop-core: provider-agnostic message model, tool trait, and the agentic loop.

pub mod message;
pub mod tool;
pub mod provider;
pub mod agent;
pub mod permission;

pub use agent::{Agent, AgentConfig, AgentEvent};
pub use message::{ContentBlock, Message, Role, StopReason};
pub use permission::{Permission, PermissionPolicy};
pub use provider::{Provider, ProviderRequest, ProviderResponse, ToolSpec};
pub use tool::{Tool, ToolContext, ToolError, ToolRegistry, ToolResult};
