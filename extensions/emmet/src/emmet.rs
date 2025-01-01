use editsync_extension_api::{self as editsync, Result};
use std::{env, fs};

struct EmmetExtension {
    did_find_server: bool,
}

const SERVER_PATH: &str = "node_modules/.bin/emmet-language-server";
const PACKAGE_NAME: &str = "@olrtg/emmet-language-server";

impl EmmetExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(
        &mut self,
        language_server_id: &editsync::LanguageServerId,
    ) -> Result<String> {
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

impl editsync::Extension for EmmetExtension {
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
}

editsync::register_extension!(EmmetExtension);
