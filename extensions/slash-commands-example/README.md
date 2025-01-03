# Slash Commands Example Extension

This is an example extension showcasing how to write slash commands.

See: [Extensions: Slash Commands](https://editsync.khulnasoft.com/docs/extensions/slash-commands) in the Editsync Docs.

## Pre-requisites

[Install Rust Toolchain](https://www.rust-lang.org/tools/install):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Setup

```sh
git clone https://github.com/khulnasoft/editsync.git
cp -RL editsync/extensions/slash-commands-example .

cd slash-commands-example/

sed -i '' '/\[lints]/,/^$/s/^workspace/#&/' Cargo.toml
curl -O https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-APACHE
echo "# Editsync Slash Commands Example Extension" > README.md
echo "Cargo.lock" > .gitignore
echo "target/" >> .gitignore
echo "*.wasm" >> .gitignore

git init
git add .
git commit -m "Initial commit"

cd ..
mv slash-commands-example MY-SUPER-COOL-EDITSYNC-EXTENSION
editsync $_
```

## Installation

1. Open the command palette (`cmd-shift-p` or `ctrl-shift-p`).
2. Launch `editsync: install dev extension`
3. Select the `slash-commands-example` folder created above

## Test

Open the assistant and type `/echo` and `/pick-one` at the beginning of a line.

## Customization

Open the `extensions.toml` file and set the `id`, `name`, `description`, `authors` and `repository` fields.

Rename `slash-commands-example.rs` you'll also have to update `Cargo.toml

## Rebuild

Rebuild to see these changes reflected:

1. Open Editsync Extensions (`cmd-shift-x` or `ctrl-shift-x`).
2. Click `Rebuild` next to your Dev Extension (formerly "Slash Command Example")

## Troubleshooting / Logs

- MacOS: `tail -f ~/Library/Logs/Editsync/Editsync.log`
- Linux: `tail -f ~/.local/share/editsync/logs/Editsync.log`

## Documentation

- [editsync.khulnasoft.com docs: Extensions: Developing Extensions](https://editsync.khulnasoft.com/docs/extensions/developing-extensions)
- [editsync.khulnasoft.com docs: Extensions: Slash Commands](https://editsync.khulnasoft.com/docs/extensions/slash-commands)
