#!/usr/bin/env sh
set -eu

# Downloads the latest tarball from https://editsync.khulnasoft.com/releases and unpacks it
# into ~/.local/. If you'd prefer to do this manually, instructions are at
# https://editsync.khulnasoft.com/docs/linux.

main() {
    platform="$(uname -s)"
    arch="$(uname -m)"
    channel="${EDITSYNC_CHANNEL:-stable}"
    temp="$(mktemp -d "/tmp/editsync-XXXXXX")"

    if [ "$platform" = "Darwin" ]; then
        platform="macos"
    elif [ "$platform" = "Linux" ]; then
        platform="linux"
    else
        echo "Unsupported platform $platform"
        exit 1
    fi

    case "$platform-$arch" in
        macos-arm64* | linux-arm64* | linux-armhf | linux-aarch64)
            arch="aarch64"
            ;;
        macos-x86* | linux-x86* | linux-i686*)
            arch="x86_64"
            ;;
        *)
            echo "Unsupported platform or architecture"
            exit 1
            ;;
    esac

    if command -v curl >/dev/null 2>&1; then
        curl () {
            command curl -fL "$@"
        }
    elif command -v wget >/dev/null 2>&1; then
        curl () {
            wget -O- "$@"
        }
    else
        echo "Could not find 'curl' or 'wget' in your path"
        exit 1
    fi

    "$platform" "$@"

    if [ "$(command -v editsync)" = "$HOME/.local/bin/editsync" ]; then
        echo "Editsync has been installed. Run with 'editsync'"
    else
        echo "To run Editsync from your terminal, you must add ~/.local/bin to your PATH"
        echo "Run:"

        case "$SHELL" in
            *zsh)
                echo "   echo 'export PATH=\$HOME/.local/bin:\$PATH' >> ~/.zshrc"
                echo "   source ~/.zshrc"
                ;;
            *fish)
                echo "   fish_add_path -U $HOME/.local/bin"
                ;;
            *)
                echo "   echo 'export PATH=\$HOME/.local/bin:\$PATH' >> ~/.bashrc"
                echo "   source ~/.bashrc"
                ;;
        esac

        echo "To run Editsync now, '~/.local/bin/editsync'"
    fi
}

linux() {
    if [ -n "${EDITSYNC_BUNDLE_PATH:-}" ]; then
        cp "$EDITSYNC_BUNDLE_PATH" "$temp/editsync-linux-$arch.tar.gz"
    else
        echo "Downloading Editsync"
        curl "https://editsync.khulnasoft.com/api/releases/$channel/latest/editsync-linux-$arch.tar.gz" > "$temp/editsync-linux-$arch.tar.gz"
    fi

    suffix=""
    if [ "$channel" != "stable" ]; then
        suffix="-$channel"
    fi

    appid=""
    case "$channel" in
      stable)
        appid="dev.editsync.Editsync"
        ;;
      nightly)
        appid="dev.editsync.Editsync-Nightly"
        ;;
      preview)
        appid="dev.editsync.Editsync-Preview"
        ;;
      dev)
        appid="dev.editsync.Editsync-Dev"
        ;;
      *)
        echo "Unknown release channel: ${channel}. Using stable app ID."
        appid="dev.editsync.Editsync"
        ;;
    esac

    # Unpack
    rm -rf "$HOME/.local/editsync$suffix.app"
    mkdir -p "$HOME/.local/editsync$suffix.app"
    tar -xzf "$temp/editsync-linux-$arch.tar.gz" -C "$HOME/.local/"

    # Setup ~/.local directories
    mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"

    # Link the binary
    if [ -f "$HOME/.local/editsync$suffix.app/bin/editsync" ]; then
        ln -sf "$HOME/.local/editsync$suffix.app/bin/editsync" "$HOME/.local/bin/editsync"
    else
        # support for versions before 0.139.x.
        ln -sf "$HOME/.local/editsync$suffix.app/bin/cli" "$HOME/.local/bin/editsync"
    fi

    # Copy .desktop file
    desktop_file_path="$HOME/.local/share/applications/${appid}.desktop"
    cp "$HOME/.local/editsync$suffix.app/share/applications/editsync$suffix.desktop" "${desktop_file_path}"
    sed -i "s|Icon=editsync|Icon=$HOME/.local/editsync$suffix.app/share/icons/hicolor/512x512/apps/editsync.png|g" "${desktop_file_path}"
    sed -i "s|Exec=editsync|Exec=$HOME/.local/editsync$suffix.app/bin/editsync|g" "${desktop_file_path}"
}

macos() {
    echo "Downloading Editsync"
    curl "https://editsync.khulnasoft.com/api/releases/$channel/latest/Editsync-$arch.dmg" > "$temp/Editsync-$arch.dmg"
    hdiutil attach -quiet "$temp/Editsync-$arch.dmg" -mountpoint "$temp/mount"
    app="$(cd "$temp/mount/"; echo *.app)"
    echo "Installing $app"
    if [ -d "/Applications/$app" ]; then
        echo "Removing existing $app"
        rm -rf "/Applications/$app"
    fi
    ditto "$temp/mount/$app" "/Applications/$app"
    hdiutil detach -quiet "$temp/mount"

    mkdir -p "$HOME/.local/bin"
    # Link the binary
    ln -sf "/Applications/$app/Contents/MacOS/cli" "$HOME/.local/bin/editsync"
}

main "$@"
