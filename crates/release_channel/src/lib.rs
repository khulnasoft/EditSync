//! Provides constructs for the Editsync app version and release channel.

#![deny(missing_docs)]

use std::{env, str::FromStr, sync::LazyLock};

use gpui::{AppContext, Global, SemanticVersion};

/// stable | dev | nightly | preview
pub static RELEASE_CHANNEL_NAME: LazyLock<String> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        env::var("EDITSYNC_RELEASE_CHANNEL")
            .unwrap_or_else(|_| include_str!("../../editsync/RELEASE_CHANNEL").trim().to_string())
    } else {
        include_str!("../../editsync/RELEASE_CHANNEL").trim().to_string()
    }
});

#[doc(hidden)]
pub static RELEASE_CHANNEL: LazyLock<ReleaseChannel> =
    LazyLock::new(|| match ReleaseChannel::from_str(&RELEASE_CHANNEL_NAME) {
        Ok(channel) => channel,
        _ => panic!("invalid release channel {}", *RELEASE_CHANNEL_NAME),
    });

/// The Git commit SHA that Editsync was built at.
#[derive(Clone)]
pub struct AppCommitSha(pub String);

struct GlobalAppCommitSha(AppCommitSha);

impl Global for GlobalAppCommitSha {}

impl AppCommitSha {
    /// Returns the global [`AppCommitSha`], if one is set.
    pub fn try_global(cx: &AppContext) -> Option<AppCommitSha> {
        cx.try_global::<GlobalAppCommitSha>()
            .map(|sha| sha.0.clone())
    }

    /// Sets the global [`AppCommitSha`].
    pub fn set_global(sha: AppCommitSha, cx: &mut AppContext) {
        cx.set_global(GlobalAppCommitSha(sha))
    }
}

struct GlobalAppVersion(SemanticVersion);

impl Global for GlobalAppVersion {}

/// The version of Editsync.
pub struct AppVersion;

impl AppVersion {
    /// Initializes the global [`AppVersion`].
    ///
    /// Attempts to read the version number from the following locations, in order:
    /// 1. the `EDITSYNC_APP_VERSION` environment variable,
    /// 2. the [`AppContext::app_metadata`],
    /// 3. the passed in `pkg_version`.
    pub fn init(pkg_version: &str) -> SemanticVersion {
        if let Ok(from_env) = env::var("EDITSYNC_APP_VERSION") {
            from_env.parse().expect("invalid EDITSYNC_APP_VERSION")
        } else {
            pkg_version.parse().expect("invalid version in Cargo.toml")
        }
    }

    /// Returns the global version number.
    pub fn global(cx: &AppContext) -> SemanticVersion {
        if cx.has_global::<GlobalAppVersion>() {
            cx.global::<GlobalAppVersion>().0
        } else {
            SemanticVersion::default()
        }
    }
}

/// A Editsync release channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ReleaseChannel {
    /// The development release channel.
    ///
    /// Used for local debug builds of Editsync.
    #[default]
    Dev,

    /// The Nightly release channel.
    Nightly,

    /// The Preview release channel.
    Preview,

    /// The Stable release channel.
    Stable,
}

struct GlobalReleaseChannel(ReleaseChannel);

impl Global for GlobalReleaseChannel {}

/// Initializes the release channel.
pub fn init(app_version: SemanticVersion, cx: &mut AppContext) {
    cx.set_global(GlobalAppVersion(app_version));
    cx.set_global(GlobalReleaseChannel(*RELEASE_CHANNEL))
}

impl ReleaseChannel {
    /// Returns the global [`ReleaseChannel`].
    pub fn global(cx: &AppContext) -> Self {
        cx.global::<GlobalReleaseChannel>().0
    }

    /// Returns the global [`ReleaseChannel`], if one is set.
    pub fn try_global(cx: &AppContext) -> Option<Self> {
        cx.try_global::<GlobalReleaseChannel>()
            .map(|channel| channel.0)
    }

    /// Returns whether we want to poll for updates for this [`ReleaseChannel`]
    pub fn poll_for_updates(&self) -> bool {
        !matches!(self, ReleaseChannel::Dev)
    }

    /// Returns the display name for this [`ReleaseChannel`].
    pub fn display_name(&self) -> &'static str {
        match self {
            ReleaseChannel::Dev => "Editsync Dev",
            ReleaseChannel::Nightly => "Editsync Nightly",
            ReleaseChannel::Preview => "Editsync Preview",
            ReleaseChannel::Stable => "Editsync",
        }
    }

    /// Returns the programmatic name for this [`ReleaseChannel`].
    pub fn dev_name(&self) -> &'static str {
        match self {
            ReleaseChannel::Dev => "dev",
            ReleaseChannel::Nightly => "nightly",
            ReleaseChannel::Preview => "preview",
            ReleaseChannel::Stable => "stable",
        }
    }

    /// Returns the application ID that's used by Wayland as application ID
    /// and WM_CLASS on X11.
    /// This also has to match the bundle identifier for Editsync on macOS.
    pub fn app_id(&self) -> &'static str {
        match self {
            ReleaseChannel::Dev => "dev.editsync.Editsync-Dev",
            ReleaseChannel::Nightly => "dev.editsync.Editsync-Nightly",
            ReleaseChannel::Preview => "dev.editsync.Editsync-Preview",
            ReleaseChannel::Stable => "dev.editsync.Editsync",
        }
    }

    /// Returns the query parameter for this [`ReleaseChannel`].
    pub fn release_query_param(&self) -> Option<&'static str> {
        match self {
            Self::Dev => None,
            Self::Nightly => Some("nightly=1"),
            Self::Preview => Some("preview=1"),
            Self::Stable => None,
        }
    }
}

/// Error indicating that release channel string does not match any known release channel names.
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct InvalidReleaseChannel;

impl FromStr for ReleaseChannel {
    type Err = InvalidReleaseChannel;

    fn from_str(channel: &str) -> Result<Self, Self::Err> {
        Ok(match channel {
            "dev" => ReleaseChannel::Dev,
            "nightly" => ReleaseChannel::Nightly,
            "preview" => ReleaseChannel::Preview,
            "stable" => ReleaseChannel::Stable,
            _ => return Err(InvalidReleaseChannel),
        })
    }
}