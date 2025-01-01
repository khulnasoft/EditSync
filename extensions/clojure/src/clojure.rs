use editsync_extension_api::{self as editsync, LanguageServerId, Result};
use std::fs;

struct ClojureExtension {
    cached_binary_path: Option<String>,
}

impl ClojureExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("clojure-lsp") {
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
            "clojure-lsp/clojure-lsp",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let asset_name = format!(
            "clojure-lsp-native-{os}-{arch}.zip",
            os = match platform {
                editsync::Os::Mac => "macos",
                editsync::Os::Linux => "linux",
                editsync::Os::Windows => "windows",
            },
            arch = match arch {
                editsync::Architecture::Aarch64 => "aarch64",
                editsync::Architecture::X8664 => "amd64",
                editsync::Architecture::X86 =>
                    return Err(format!("unsupported architecture: {arch:?}")),
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("clojure-lsp-{}", release.version);
        let binary_path = format!("{version_dir}/clojure-lsp");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            editsync::set_language_server_installation_status(
                language_server_id,
                &editsync::LanguageServerInstallationStatus::Downloading,
            );

            editsync::download_file(
                &asset.download_url,
                &version_dir,
                editsync::DownloadedFileType::Zip,
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

impl editsync::Extension for ClojureExtension {
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
            args: Vec::new(),
            env: Default::default(),
        })
    }
}

editsync::register_extension!(ClojureExtension);
