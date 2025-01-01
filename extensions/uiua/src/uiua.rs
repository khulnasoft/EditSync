use editsync_extension_api::{self as editsync, Result};

struct UiuaExtension;

impl editsync::Extension for UiuaExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        let path = worktree
            .which("uiua")
            .ok_or_else(|| "uiua is not installed".to_string())?;

        Ok(editsync::Command {
            command: path,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

editsync::register_extension!(UiuaExtension);
