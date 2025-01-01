# Getting Started

Welcome to Editsync! We are excited to have you. Here is a jumping-off point to getting started.

## Download Editsync

### macOS

You can obtain the stable builds via the [download page](https://editsync.khulnasoft.com/download). If you want to download our preview build, you can find it on its [releases page](https://editsync.khulnasoft.com/releases/preview) After the first manual installation, Editsync will periodically check for and install updates automatically for you.

You can also install Editsync stable via Homebrew:

```sh
brew install --cask editsync
```

As well as Editsync preview:

```sh
brew install --cask editsync@preview
```

### Linux

For most people, the easiest way to install Editsync is through our installation script:

```sh
curl -f https://editsync.khulnasoft.com/install.sh | sh
```

If you'd like to help us test our new features, you can also install our preview build:

```sh
curl -f https://editsync.khulnasoft.com/install.sh | EDITSYNC_CHANNEL=preview sh
```

This script supports `x86_64` and `AArch64`, as well as common Linux distributions: Ubuntu, Arch, Debian, RedHat, CentOS, Fedora, and more.

If this script is insufficient for your use case or you run into problems running Editsync, please see our [Linux-specific documentation](./linux.md).

## Command Palette

The Command Palette is the main way to access functionality in Editsync, and its keybinding is the first one you should make yourself familiar with.

To open the Command Palette, use {#kb command_palette::Toggle}.

The Command Palette allows you to access pretty much any functionality that's available in Editsync.

![The opened Command Palette](https://editsync.khulnasoft.com/img/features/command-palette.jpg)

Try it! Open the Command Palette and type in `new file`. You should see the list of commands being filtered down to `workspace: new file`. Hit return and you end up with a new buffer!

Any time you see instructions that include commands of the form `editsync: ...` or `editor: ...` and so on that means you need to execute them in the Command Palette.

## Configure Editsync

Use {#kb editsync::OpenSettings} to open your custom settings to set things like fonts, formatting settings, per-language settings, and more.

On macOS, you can access the default configuration using the `Editsync > Settings > Open Default Settings` menu item. See [Configuring Editsync](./configuring-editsync.md) for all available settings.

On Linux, you can access the default configuration via the Command Palette. Open it with {#kb editsync::OpenDefaultSettings} and type in `editsync: open default settings` and then hit return.

## Set up your key bindings

On macOS, you can access the default key binding set using the `Editsync > Settings > Open Default Key Bindings` menu item. Use <kbd>cmd-k cmd-s|ctrl-k ctrl-s</kbd> to open your custom keymap to add your key bindings. See Key Bindings for more info.

On Linux, you can access the default key bindings via the Command Palette. Open it with <kbd>ctrl-shift-p</kbd> and type in `editsync: open default keymap` and then hit return.
