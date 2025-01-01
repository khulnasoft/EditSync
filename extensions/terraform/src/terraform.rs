use editsync::LanguageServerId;
use editsync_extension_api::{self as editsync, Result};
use std::fs;

struct TerraformExtension {
    cached_binary_path: Option<String>,
}

impl TerraformExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("terraform-ls") {
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
            "hashicorp/terraform-ls",
            editsync::GithubReleaseOptions {
                require_assets: false,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let download_url = format!(
            "https://releases.hashicorp.com/terraform-ls/{version}/terraform-ls_{version}_{os}_{arch}.zip",
            version = release.version.strip_prefix('v').unwrap_or(&release.version),
            os = match platform {
                editsync::Os::Mac => "darwin",
                editsync::Os::Linux => "linux",
                editsync::Os::Windows => "windows",
            },
            arch = match arch {
                editsync::Architecture::Aarch64 => "arm64",
                editsync::Architecture::X86 => "386",
                editsync::Architecture::X8664 => "amd64",
            },
        );

        let version_dir = format!("terraform-ls-{}", release.version);
        let binary_path = format!("{version_dir}/terraform-ls");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );

            editsync::download_file(
                &download_url,
                &version_dir,
                editsync::DownloadedFileType::Zip,
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
        Ok(binary_path)
    }
}

impl editsync::Extension for TerraformExtension {
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
        Ok(editsync::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["serve".to_string()],
            env: Default::default(),
        })
    }
}

editsync::register_extension!(TerraformExtension);
