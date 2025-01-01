use std::fs;

use editsync_extension_api::{self as editsync, LanguageServerId, Result};

pub struct ErlangLanguagePlatform {
    cached_binary_path: Option<String>,
}

impl ErlangLanguagePlatform {
    pub const LANGUAGE_SERVER_ID: &'static str = "elp";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        Ok(editsync::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["server".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("elp") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        editsync::set_language_server_installation_status(
            language_server_id,
            &editsync::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = editsync::latest_github_release(
            "WhatsApp/erlang-language-platform",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let asset_name = {
            let otp_version = "26.2";
            let (os, os_target) = match platform {
                editsync::Os::Mac => ("macos", "apple-darwin"),
                editsync::Os::Linux => ("linux", "unknown-linux-gnu"),
                editsync::Os::Windows => return Err(format!("unsupported platform: {platform:?}")),
            };

            format!(
                "elp-{os}-{arch}-{os_target}-otp-{otp_version}.tar.gz",
                arch = match arch {
                    editsync::Architecture::Aarch64 => "aarch64",
                    editsync::Architecture::X8664 => "x86_64",
                    editsync::Architecture::X86 =>
                        return Err(format!("unsupported architecture: {arch:?}")),
                },
            )
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("elp-{}", release.version);
        let binary_path = format!("{version_dir}/elp");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );

            editsync::download_file(
                &asset.download_url,
                &version_dir,
                editsync::DownloadedFileType::GzipTar,
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
        Ok(binary_path)
    }
}
