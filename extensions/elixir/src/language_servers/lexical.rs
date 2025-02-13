use std::fs;

use editsync::lsp::{Completion, CompletionKind, Symbol};
use editsync::{CodeLabel, CodeLabelSpan, LanguageServerId};
use editsync_extension_api::settings::LspSettings;
use editsync_extension_api::{self as editsync, Result};

pub struct LexicalBinary {
    pub path: String,
    pub args: Option<Vec<String>>,
}

pub struct Lexical {
    cached_binary_path: Option<String>,
}

impl Lexical {
    pub const LANGUAGE_SERVER_ID: &'static str = "lexical";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<LexicalBinary> {
        let binary_settings = LspSettings::for_worktree("lexical", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(LexicalBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = worktree.which("lexical") {
            return Ok(LexicalBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(LexicalBinary {
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
            "lexical-lsp/lexical",
            editsync::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = format!("lexical-{version}.zip", version = release.version);

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("lexical-{}", release.version);
        let binary_path = format!("{version_dir}/lexical/bin/start_lexical.sh");

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

            editsync::make_file_executable(&binary_path)?;
            editsync::make_file_executable(&format!("{version_dir}/lexical/bin/debug_shell.sh"))?;
            editsync::make_file_executable(&format!("{version_dir}/lexical/priv/port_wrapper.sh"))?;

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
        Ok(LexicalBinary {
            path: binary_path,
            args: binary_args,
        })
    }

    pub fn label_for_completion(&self, completion: Completion) -> Option<CodeLabel> {
        match completion.kind? {
            CompletionKind::Module
            | CompletionKind::Class
            | CompletionKind::Interface
            | CompletionKind::Struct => {
                let name = completion.label;
                let defmodule = "defmodule ";
                let code = format!("{defmodule}{name}");

                Some(CodeLabel {
                    code,
                    spans: vec![CodeLabelSpan::code_range(
                        defmodule.len()..defmodule.len() + name.len(),
                    )],
                    filter_range: (0..name.len()).into(),
                })
            }
            CompletionKind::Function | CompletionKind::Constant => {
                let name = completion.label;
                let def = "def ";
                let code = format!("{def}{name}");

                Some(CodeLabel {
                    code,
                    spans: vec![CodeLabelSpan::code_range(def.len()..def.len() + name.len())],
                    filter_range: (0..name.len()).into(),
                })
            }
            CompletionKind::Operator => {
                let name = completion.label;
                let def_a = "def a ";
                let code = format!("{def_a}{name} b");

                Some(CodeLabel {
                    code,
                    spans: vec![CodeLabelSpan::code_range(
                        def_a.len()..def_a.len() + name.len(),
                    )],
                    filter_range: (0..name.len()).into(),
                })
            }
            _ => None,
        }
    }

    pub fn label_for_symbol(&self, _symbol: Symbol) -> Option<CodeLabel> {
        None
    }
}
