use editsync_extension_api::{
    self as editsync, serde_json, settings::LspSettings, LanguageServerId, Result,
};
use std::fs;

struct ZigExtension {
    cached_binary_path: Option<String>,
}

#[derive(Clone)]
struct ZlsBinary {
    path: String,
    args: Option<Vec<String>>,
    environment: Option<Vec<(String, String)>>,
}

impl ZigExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<ZlsBinary> {
        let mut args: Option<Vec<String>> = None;

        let (platform, arch) = editsync::current_platform();
        let environment = match platform {
            editsync::Os::Mac | editsync::Os::Linux => Some(worktree.shell_env()),
            editsync::Os::Windows => None,
        };

        if let Ok(lsp_settings) = LspSettings::for_worktree("zls", worktree) {
            if let Some(binary) = lsp_settings.binary {
                args = binary.arguments;
                if let Some(path) = binary.path {
                    return Ok(ZlsBinary {
                        path: path.clone(),
                        args,
                        environment,
                    });
                }
            }
        }

        if let Some(path) = worktree.which("zls") {
            return Ok(ZlsBinary {
                path,
                args,
                environment,
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(ZlsBinary {
                    path: path.clone(),
                    args,
                    environment,
                });
            }
        }

        editsync::set_language_server_installation_status(
            language_server_id,
            &editsync::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        // Note that in github releases and on zlstools.org the tar.gz asset is not shown
        // but is available at https://builds.zigtools.org/zls-{os}-{arch}-{version}.tar.gz
        let release = editsync::latest_github_release(
            "zigtools/zls",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let arch: &str = match arch {
            editsync::Architecture::Aarch64 => "aarch64",
            editsync::Architecture::X86 => "x86",
            editsync::Architecture::X8664 => "x86_64",
        };

        let os: &str = match platform {
            editsync::Os::Mac => "macos",
            editsync::Os::Linux => "linux",
            editsync::Os::Windows => "windows",
        };

        let extension: &str = match platform {
            editsync::Os::Mac | editsync::Os::Linux => "tar.gz",
            editsync::Os::Windows => "zip",
        };

        let asset_name: String = format!("zls-{}-{}-{}.{}", os, arch, release.version, extension);
        let download_url = format!("https://builds.zigtools.org/{}", asset_name);

        let version_dir = format!("zls-{}", release.version);
        let binary_path = match platform {
            editsync::Os::Mac | editsync::Os::Linux => format!("{version_dir}/zls"),
            editsync::Os::Windows => format!("{version_dir}/zls.exe"),
        };

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );

            editsync::download_file(
                &download_url,
                &version_dir,
                match platform {
                    editsync::Os::Mac | editsync::Os::Linux => {
                        editsync::DownloadedFileType::GzipTar
                    }
                    editsync::Os::Windows => editsync::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            editsync::make_file_executable(&binary_path)?;

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
        Ok(ZlsBinary {
            path: binary_path,
            args,
            environment,
        })
    }
}

impl editsync::Extension for ZigExtension {
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
        let zls_binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(editsync::Command {
            command: zls_binary.path,
            args: zls_binary.args.unwrap_or_default(),
            env: zls_binary.environment.unwrap_or_default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("zls", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

editsync::register_extension!(ZigExtension);
