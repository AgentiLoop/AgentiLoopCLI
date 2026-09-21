//! Built-in tools: read_file, write_file, edit_file, list_dir, bash.

mod fs;
mod shell;

use agentiloop_core::ToolRegistry;

pub use fs::{EditFile, ListDir, ReadFile, WriteFile};
pub use shell::Bash;

/// Registry pre-populated with every built-in tool.
pub fn default_registry() -> ToolRegistry {
    let mut r = ToolRegistry::new();
    r.register(ReadFile)
        .register(WriteFile)
        .register(EditFile)
        .register(ListDir)
        .register(Bash);
    r
}
