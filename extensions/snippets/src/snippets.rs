use editsync::LanguageServerId;
use editsync_extension_api::{self as editsync, settings::LspSettings, Result};
use serde_json::json;
use std::fs;

struct SnippetExtension {
    cached_binary_path: Option<String>,
}

impl SnippetExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("simple-completion-language-server") {
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
            "khulnasoft/simple-completion-language-server",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let asset_name = format!(
            "simple-completion-language-server-{arch}-{os}.tar.gz",
            arch = match arch {
                editsync::Architecture::Aarch64 => "aarch64",
                editsync::Architecture::X86 => "x86",
                editsync::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                editsync::Os::Mac => "apple-darwin",
                editsync::Os::Linux => "unknown-linux-gnu",
                editsync::Os::Windows => "pc-windows-msvc",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("simple-completion-language-server-{}", release.version);
        let binary_path = format!("{version_dir}/simple-completion-language-server");

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

impl editsync::Extension for SnippetExtension {
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
            args: vec![],
            env: vec![("SCLS_CONFIG_SUBDIRECTORY".to_owned(), "editsync".to_owned())],
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &editsync_extension_api::Worktree,
    ) -> Result<Option<editsync_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_else(|| {
                json!({
                    "max_completion_items": 20,
                    "snippets_first": true,
                    "feature_words": false,
                    "feature_snippets": true,
                    "feature_paths": true
                })
            });
        Ok(Some(settings))
    }
}

editsync::register_extension!(SnippetExtension);
