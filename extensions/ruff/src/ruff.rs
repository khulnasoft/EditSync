use std::fs;
use editsync::LanguageServerId;
use editsync_extension_api::{self as editsync, settings::LspSettings, Result};

struct RuffBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct RuffExtension {
    cached_binary_path: Option<String>,
}

impl RuffExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<RuffBinary> {
        let binary_settings = LspSettings::for_worktree("ruff", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(RuffBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = worktree.which("ruff") {
            return Ok(RuffBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(RuffBinary {
                    path: path.clone(),
                    args: binary_args,
                });
            }
        }

        editsync::set_language_server_installation_status(
            language_server_id,
            &editsync::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = editsync::latest_github_release(
            "astral-sh/ruff",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();

        let asset_stem = format!(
            "ruff-{arch}-{os}",
            arch = match arch {
                editsync::Architecture::Aarch64 => "aarch64",
                editsync::Architecture::X86 => "x86",
                editsync::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                editsync::Os::Mac => "apple-darwin",
                editsync::Os::Linux => "unknown-linux-gnu",
                editsync::Os::Windows => "pc-windows-msvc",
            }
        );
        let asset_name = format!(
            "{asset_stem}.{suffix}",
            suffix = match platform {
                editsync::Os::Windows => "zip",
                _ => "tar.gz",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("ruff-{}", release.version);
        let binary_path = match platform {
            editsync::Os::Windows => format!("{version_dir}/ruff.exe"),
            _ => format!("{version_dir}/{asset_stem}/ruff"),
        };

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );
            let file_kind = match platform {
                editsync::Os::Windows => editsync::DownloadedFileType::Zip,
                _ => editsync::DownloadedFileType::GzipTar,
            };
            editsync::download_file(&asset.download_url, &version_dir, file_kind)
                .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(RuffBinary {
            path: binary_path,
            args: binary_args,
        })
    }
}

impl editsync::Extension for RuffExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        let ruff_binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(editsync::Command {
            command: ruff_binary.path,
            args: ruff_binary.args.unwrap_or_else(|| vec!["server".into()]),
            env: vec![],
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &editsync_extension_api::Worktree,
    ) -> Result<Option<editsync_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &editsync_extension_api::Worktree,
    ) -> Result<Option<editsync_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

editsync::register_extension!(RuffExtension);
