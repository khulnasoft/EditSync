use std::{env, fs};
use editsync::{
    serde_json::{self, Value},
    settings::LspSettings,
};
use editsync_extension_api::{self as editsync, Result};

const SERVER_PATH: &str = "node_modules/@elm-tooling/elm-language-server/out/node/index.js";
const PACKAGE_NAME: &str = "@elm-tooling/elm-language-server";

struct ElmExtension {
    did_find_server: bool,
}

impl ElmExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(&mut self, server_id: &editsync::LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        editsync::set_language_server_installation_status(
            server_id,
            &editsync::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = editsync::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || editsync::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            editsync::set_language_server_installation_status(
                server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );
            let result = editsync::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }
}

impl editsync::Extension for ElmExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        server_id: &editsync::LanguageServerId,
        _worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        let server_path = self.server_script_path(server_id)?;
        Ok(editsync::Command {
            command: editsync::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<Option<Value>> {
        // elm-language-server expects workspace didChangeConfiguration notification
        // params to be the same as lsp initialization_options
        let initialization_options = LspSettings::for_worktree(server_id.as_ref(), worktree)?
            .initialization_options
            .clone()
            .unwrap_or_default();

        Ok(Some(match initialization_options.clone().as_object_mut() {
            Some(op) => {
                // elm-language-server requests workspace configuration
                // for the `elmLS` section, so we have to nest
                // another copy of initialization_options there
                op.insert("elmLS".into(), initialization_options);
                serde_json::to_value(op).unwrap_or_default()
            }
            None => initialization_options,
        }))
    }
}

editsync::register_extension!(ElmExtension);
