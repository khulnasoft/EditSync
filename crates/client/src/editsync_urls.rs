//! Contains helper functions for constructing URLs to various Editsync-related pages.
//!
//! These URLs will adapt to the configured server URL in order to construct
//! links appropriate for the environment (e.g., by linking to a local copy of
//! editsync.khulnasoft.com in development).

use gpui::AppContext;
use settings::Settings;

use crate::ClientSettings;

fn server_url(cx: &AppContext) -> &str {
    &ClientSettings::get_global(cx).server_url
}

/// Returns the URL to the account page on editsync.khulnasoft.com.
pub fn account_url(cx: &AppContext) -> String {
    format!("{server_url}/account", server_url = server_url(cx))
}
