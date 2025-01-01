#!/usr/bin/env sh
set -eu

# Uninstalls Editsync that was installed using the install.sh script

check_remaining_installations() {
    platform="$(uname -s)"
    if [ "$platform" = "Darwin" ]; then
        # Check for any Editsync variants in /Applications
        remaining=$(ls -d /Applications/Editsync*.app 2>/dev/null | wc -l)
        [ "$remaining" -eq 0 ]
    else
        # Check for any Editsync variants in ~/.local
        remaining=$(ls -d "$HOME/.local/editsync"*.app 2>/dev/null | wc -l)
        [ "$remaining" -eq 0 ]
    fi
}

prompt_remove_preferences() {
    printf "Do you want to keep your Editsync preferences? [Y/n] "
    read -r response
    case "$response" in
        [nN]|[nN][oO])
            rm -rf "$HOME/.config/editsync"
            echo "Preferences removed."
            ;;
        *)
            echo "Preferences kept."
            ;;
    esac
}

main() {
    platform="$(uname -s)"
    channel="${EDITSYNC_CHANNEL:-stable}"

    if [ "$platform" = "Darwin" ]; then
        platform="macos"
    elif [ "$platform" = "Linux" ]; then
        platform="linux"
    else
        echo "Unsupported platform $platform"
        exit 1
    fi

    "$platform"

    echo "Editsync has been uninstalled"
}

linux() {
    suffix=""
    if [ "$channel" != "stable" ]; then
        suffix="-$channel"
    fi

    appid=""
    db_suffix="stable"
    case "$channel" in
      stable)
        appid="dev.editsync.Editsync"
        db_suffix="stable"
        ;;
      nightly)
        appid="dev.editsync.Editsync-Nightly"
        db_suffix="nightly"
        ;;
      preview)
        appid="dev.editsync.Editsync-Preview"
        db_suffix="preview"
        ;;
      dev)
        appid="dev.editsync.Editsync-Dev"
        db_suffix="dev"
        ;;
      *)
        echo "Unknown release channel: ${channel}. Using stable app ID."
        appid="dev.editsync.Editsync"
        db_suffix="stable"
        ;;
    esac

    # Remove the app directory
    rm -rf "$HOME/.local/editsync$suffix.app"

    # Remove the binary symlink
    rm -f "$HOME/.local/bin/editsync"

    # Remove the .desktop file
    rm -f "$HOME/.local/share/applications/${appid}.desktop"

    # Remove the database directory for this channel
    rm -rf "$HOME/.local/share/editsync/db/0-$db_suffix"

    # Remove socket file
    rm -f "$HOME/.local/share/editsync/editsync-$db_suffix.sock"

    # Remove the entire Editsync directory if no installations remain
    if check_remaining_installations; then
        rm -rf "$HOME/.local/share/editsync"
        prompt_remove_preferences
    fi

    rm -rf $HOME/.editsync_server
}

macos() {
    app="Editsync.app"
    db_suffix="stable"
    app_id="dev.editsync.Editsync"
    case "$channel" in
      nightly)
        app="Editsync Nightly.app"
        db_suffix="nightly"
        app_id="dev.editsync.Editsync-Nightly"
        ;;
      preview)
        app="Editsync Preview.app"
        db_suffix="preview"
        app_id="dev.editsync.Editsync-Preview"
        ;;
      dev)
        app="Editsync Dev.app"
        db_suffix="dev"
        app_id="dev.editsync.Editsync-Dev"
        ;;
    esac

    # Remove the app bundle
    if [ -d "/Applications/$app" ]; then
        rm -rf "/Applications/$app"
    fi

    # Remove the binary symlink
    rm -f "$HOME/.local/bin/editsync"

    # Remove the database directory for this channel
    rm -rf "$HOME/Library/Application Support/Editsync/db/0-$db_suffix"

    # Remove app-specific files and directories
    rm -rf "$HOME/Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments/$app_id.sfl"*
    rm -rf "$HOME/Library/Caches/$app_id"
    rm -rf "$HOME/Library/HTTPStorages/$app_id"
    rm -rf "$HOME/Library/Preferences/$app_id.plist"
    rm -rf "$HOME/Library/Saved Application State/$app_id.savedState"

    # Remove the entire Editsync directory if no installations remain
    if check_remaining_installations; then
        rm -rf "$HOME/Library/Application Support/Editsync"
        rm -rf "$HOME/Library/Logs/Editsync"

        prompt_remove_preferences
    fi

    rm -rf $HOME/.editsync_server
}

main "$@"
