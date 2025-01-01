mod language_servers;

use editsync_extension_api::{self as editsync, Result};

use crate::language_servers::{ErlangLanguagePlatform, ErlangLs};

struct ErlangExtension {
    erlang_ls: Option<ErlangLs>,
    erlang_language_platform: Option<ErlangLanguagePlatform>,
}

impl editsync::Extension for ErlangExtension {
    fn new() -> Self {
        Self {
            erlang_ls: None,
            erlang_language_platform: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        match language_server_id.as_ref() {
            ErlangLs::LANGUAGE_SERVER_ID => {
                let erlang_ls = self.erlang_ls.get_or_insert_with(ErlangLs::new);

                Ok(editsync::Command {
                    command: erlang_ls.language_server_binary_path(language_server_id, worktree)?,
                    args: vec![],
                    env: Default::default(),
                })
            }
            ErlangLanguagePlatform::LANGUAGE_SERVER_ID => {
                let erlang_language_platform = self
                    .erlang_language_platform
                    .get_or_insert_with(ErlangLanguagePlatform::new);
                erlang_language_platform.language_server_command(language_server_id, worktree)
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }
}

editsync::register_extension!(ErlangExtension);
