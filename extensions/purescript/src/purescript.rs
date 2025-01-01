use std::{env, fs};
use editsync_extension_api::{self as editsync, serde_json, Result};

const SERVER_PATH: &str = "node_modules/.bin/purescript-language-server";
const PACKAGE_NAME: &str = "purescript-language-server";

struct PurescriptExtension {
    did_find_server: bool,
}

impl PurescriptExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &editsync::LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        editsync::set_language_server_installation_status(
            language_server_id,
            &editsync::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = editsync::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || editsync::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            editsync::set_language_server_installation_status(
                language_server_id,
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

impl editsync::Extension for PurescriptExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &editsync::LanguageServerId,
        _worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        let server_path = self.server_script_path(language_server_id)?;
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

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &editsync::LanguageServerId,
        _worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "purescript": {
                "addSpagoSources": true
            }
        })))
    }
}

editsync::register_extension!(PurescriptExtension);