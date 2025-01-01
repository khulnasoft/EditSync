use editsync_extension_api::{self as editsync, settings::LspSettings, LanguageServerId, Result};
use std::fs;

struct OmnisharpBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct CsharpExtension {
    cached_binary_path: Option<String>,
}

impl CsharpExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<OmnisharpBinary> {
        let binary_settings = LspSettings::for_worktree("omnisharp", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(OmnisharpBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = worktree.which("OmniSharp") {
            return Ok(OmnisharpBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(OmnisharpBinary {
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
            "OmniSharp/omnisharp-roslyn",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let asset_name = format!(
            "omnisharp-{os}-{arch}-net6.0.{extension}",
            os = match platform {
                editsync::Os::Mac => "osx",
                editsync::Os::Linux => "linux",
                editsync::Os::Windows => "win",
            },
            arch = match arch {
                editsync::Architecture::Aarch64 => "arm64",
                editsync::Architecture::X86 => "x86",
                editsync::Architecture::X8664 => "x64",
            },
            extension = match platform {
                editsync::Os::Mac | editsync::Os::Linux => "tar.gz",
                editsync::Os::Windows => "zip",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("omnisharp-{}", release.version);
        let binary_path = format!("{version_dir}/OmniSharp");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );

            editsync::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    editsync::Os::Mac | editsync::Os::Linux => {
                        editsync::DownloadedFileType::GzipTar
                    }
                    editsync::Os::Windows => editsync::DownloadedFileType::Zip,
                },
            )
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
        Ok(OmnisharpBinary {
            path: binary_path,
            args: binary_args,
        })
    }
}

impl editsync::Extension for CsharpExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        let omnisharp_binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(editsync::Command {
            command: omnisharp_binary.path,
            args: omnisharp_binary.args.unwrap_or_else(|| vec!["-lsp".into()]),
            env: Default::default(),
        })
    }
}

editsync::register_extension!(CsharpExtension);
