# Installing Extensions

You can search for extensions by launching the Editsync Extension Gallery by pressing `cmd-shift-x` (macOS) or `ctrl-shift-x` (Linux), opening the command palette and selecting `editsync: extensions` or by selecting "Editsync > Extensions" from the menu bar.

Here you can view the extensions that you currently have installed or search and install new ones.

## Installation Location

- On macOS, extensions are installed in `~/Library/Application Support/Editsync/extensions`.
- On Linux, they are installed in either `$XDG_DATA_HOME/editsync/extensions` or `~/.local/share/editsync/extensions`.

This directory contains two subdirectories:

- `installed`, which contains the source code for each extension.
- `work` which contains files created by the extension itself, such as downloaded language servers.

## Auto installing

To automate extension installation/uninstallation see the docs for [auto_install_extensions](../configuring-editsync.md#auto-install-extensions).
