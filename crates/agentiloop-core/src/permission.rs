use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Allow,
    Deny,
}

/// Decides whether a tool call may run. The CLI supplies an interactive
/// implementation; tests/CI can supply `AllowAll`.
#[async_trait]
pub trait PermissionPolicy: Send + Sync {
    async fn check(&self, tool: &str, is_mutating: bool, input: &Value) -> Permission;
}

pub struct AllowAll;

#[async_trait]
impl PermissionPolicy for AllowAll {
    async fn check(&self, _tool: &str, _is_mutating: bool, _input: &Value) -> Permission {
        Permission::Allow
    }
}

pub type SharedPolicy = Arc<dyn PermissionPolicy>;
