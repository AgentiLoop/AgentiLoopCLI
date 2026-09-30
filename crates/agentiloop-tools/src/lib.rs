//! Built-in tools: read_file, write_file, edit_file, list_dir, glob, grep, web_fetch, bash, and
//! apply_patch (registered for the Codex provider, whose models are trained on it).

mod fs;
mod patch;
mod search;
mod shell;
pub mod undo;
mod web;

use agentiloop_core::ToolRegistry;

pub use fs::{EditFile, ListDir, ReadFile, WriteFile};
pub use patch::ApplyPatch;
pub use search::{GlobFiles, Grep};
pub use shell::Bash;
pub use web::WebFetch;

/// Registry pre-populated with every built-in tool.
pub fn default_registry() -> ToolRegistry {
    let mut r = ToolRegistry::new();
    r.register(ReadFile)
        .register(WriteFile)
        .register(EditFile)
        .register(ListDir)
        .register(GlobFiles)
        .register(Grep)
        .register(WebFetch)
        .register(Bash);
    r
}
