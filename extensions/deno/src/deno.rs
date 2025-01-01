use std::fs;
use editsync::lsp::CompletionKind;
use editsync::{serde_json, CodeLabel, CodeLabelSpan, LanguageServerId};
use editsync_extension_api::settings::LspSettings;
use editsync_extension_api::{self as editsync, Result};

struct DenoExtension {
    cached_binary_path: Option<String>,
}

impl DenoExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("deno") {
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
            "denoland/deno",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = editsync::current_platform();
        let asset_name = format!(
            "deno-{arch}-{os}.zip",
            arch = match arch {
                editsync::Architecture::Aarch64 => "aarch64",
                editsync::Architecture::X8664 => "x86_64",
                editsync::Architecture::X86 =>
                    return Err(format!("unsupported architecture: {arch:?}")),
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

        let version_dir = format!("deno-{}", release.version);
        let binary_path = format!("{version_dir}/deno");

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

impl editsync::Extension for DenoExtension {
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
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &editsync::LanguageServerId,
        _worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "provideFormatter": true,
        })))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &editsync::LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("deno", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: editsync::lsp::Completion,
    ) -> Option<CodeLabel> {
        let highlight_name = match completion.kind? {
            CompletionKind::Class | CompletionKind::Interface | CompletionKind::Constructor => {
                "type"
            }
            CompletionKind::Constant => "constant",
            CompletionKind::Function | CompletionKind::Method => "function",
            CompletionKind::Property | CompletionKind::Field => "property",
            _ => return None,
        };

        let len = completion.label.len();
        let name_span = CodeLabelSpan::literal(completion.label, Some(highlight_name.to_string()));

        Some(editsync::CodeLabel {
            code: Default::default(),
            spans: if let Some(detail) = completion.detail {
                vec![
                    name_span,
                    CodeLabelSpan::literal(" ", None),
                    CodeLabelSpan::literal(detail, None),
                ]
            } else {
                vec![name_span]
            },
            filter_range: (0..len).into(),
        })
    }
}

editsync::register_extension!(DenoExtension);
