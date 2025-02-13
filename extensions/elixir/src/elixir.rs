mod language_servers;

use editsync::lsp::{Completion, Symbol};
use editsync::{serde_json, CodeLabel, LanguageServerId};
use editsync_extension_api::{self as editsync, Result};

use crate::language_servers::{ElixirLs, Lexical, NextLs};

struct ElixirExtension {
    elixir_ls: Option<ElixirLs>,
    next_ls: Option<NextLs>,
    lexical: Option<Lexical>,
}

impl editsync::Extension for ElixirExtension {
    fn new() -> Self {
        Self {
            elixir_ls: None,
            next_ls: None,
            lexical: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<editsync::Command> {
        match language_server_id.as_ref() {
            ElixirLs::LANGUAGE_SERVER_ID => {
                let elixir_ls = self.elixir_ls.get_or_insert_with(ElixirLs::new);

                Ok(editsync::Command {
                    command: elixir_ls.language_server_binary_path(language_server_id, worktree)?,
                    args: vec![],
                    env: Default::default(),
                })
            }
            NextLs::LANGUAGE_SERVER_ID => {
                let next_ls = self.next_ls.get_or_insert_with(NextLs::new);

                Ok(editsync::Command {
                    command: next_ls.language_server_binary_path(language_server_id, worktree)?,
                    args: vec!["--stdio".to_string()],
                    env: Default::default(),
                })
            }
            Lexical::LANGUAGE_SERVER_ID => {
                let lexical = self.lexical.get_or_insert_with(Lexical::new);
                let lexical_binary =
                    lexical.language_server_binary(language_server_id, worktree)?;

                Ok(editsync::Command {
                    command: lexical_binary.path,
                    args: lexical_binary.args.unwrap_or_default(),
                    env: Default::default(),
                })
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }

    fn label_for_completion(
        &self,
        language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        match language_server_id.as_ref() {
            ElixirLs::LANGUAGE_SERVER_ID => {
                self.elixir_ls.as_ref()?.label_for_completion(completion)
            }
            NextLs::LANGUAGE_SERVER_ID => self.next_ls.as_ref()?.label_for_completion(completion),
            Lexical::LANGUAGE_SERVER_ID => self.lexical.as_ref()?.label_for_completion(completion),
            _ => None,
        }
    }

    fn label_for_symbol(
        &self,
        language_server_id: &LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        match language_server_id.as_ref() {
            ElixirLs::LANGUAGE_SERVER_ID => self.elixir_ls.as_ref()?.label_for_symbol(symbol),
            NextLs::LANGUAGE_SERVER_ID => self.next_ls.as_ref()?.label_for_symbol(symbol),
            Lexical::LANGUAGE_SERVER_ID => self.lexical.as_ref()?.label_for_symbol(symbol),
            _ => None,
        }
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        match language_server_id.as_ref() {
            NextLs::LANGUAGE_SERVER_ID => Ok(Some(serde_json::json!({
                "experimental": {
                    "completions": {
                        "enable": true
                    }
                }
            }))),
            _ => Ok(None),
        }
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &editsync::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if language_server_id.as_ref() == ElixirLs::LANGUAGE_SERVER_ID {
            if let Some(elixir_ls) = self.elixir_ls.as_mut() {
                return elixir_ls.language_server_workspace_configuration(worktree);
            }
        }

        Ok(None)
    }
}

editsync::register_extension!(ElixirExtension);
