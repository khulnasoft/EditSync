# Developing Extensions

## Extension Capabilities

Extensions can add the following capabilities to Editsync:

- [Languages](./languages.md)
- [Themes](./themes.md)
- [Slash Commands](./slash-commands.md)
- [Context Servers](./context-servers.md)

## Developing an Extension Locally

Before starting to develop an extension for Editsync, be sure to [install Rust via rustup](https://www.rust-lang.org/tools/install).

When developing an extension, you can use it in Editsync without needing to publish it by installing it as a _dev extension_.

From the extensions page, click the `Install Dev Extension` button and select the directory containing your extension.

If you already have a published extension with the same name installed, your dev extension will override it.

## Directory Structure of a Editsync Extension

A Editsync extension is a Git repository that contains an `extension.toml`. This file must contain some
basic information about the extension:

```toml
id = "my-extension"
name = "My extension"
version = "0.0.1"
schema_version = 1
authors = ["Your Name <you@example.com>"]
description = "My cool extension"
repository = "https://github.com/your-name/my-editsync-extension"
```

In addition to this, there are several other optional files and directories that can be used to add functionality to a Editsync extension. An example directory structure of an extension that provides all capabilities is as follows:

```
my-extension/
  extension.toml
  Cargo.toml
  src/
    lib.rs
  languages/
    my-language/
      config.toml
      highlights.scm
  themes/
    my-theme.json
```

## WebAssembly

Procedural parts of extensions are written in Rust and compiled to WebAssembly. To develop an extension that includes custom code, include a `Cargo.toml` like this:

```toml
[package]
name = "my-extension"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
editsync_extension_api = "0.1.0"
```

Make sure to use the latest version of the [`editsync_extension_api`](https://crates.io/crates/editsync_extension_api) available on crates.io.

In the `src/lib.rs` file in your Rust crate you will need to define a struct for your extension and implement the `Extension` trait, as well as use the `register_extension!` macro to register your extension:

```rs
use editsync_extension_api as editsync;

struct MyExtension {
    // ... state
}

impl editsync::Extension for MyExtension {
    // ...
}

editsync::register_extension!(MyExtension);
```

## Publishing your extension

To publish an extension, open a PR to [the `khulnasoft/extensions` repo](https://github.com/khulnasoft/extensions).

> Note: It is very helpful if you fork the `khulnasoft/extensions` repo to a personal GitHub account instead of a GitHub organization, as this allows Editsync staff to push any needed changes to your PR to expedite the publishing process.

In your PR, do the following:

1. Add your extension as a Git submodule within the `extensions/` directory
2. Add a new entry to the top-level `extensions.toml` file containing your extension:

```toml
[my-extension]
submodule = "extensions/my-extension"
version = "0.0.1"
```

3. Run `pnpm sort-extensions` to ensure `extensions.toml` and `.gitmodules` are sorted

Once your PR is merged, the extension will be packaged and published to the Editsync extension registry.

> Extension IDs and names should not contain `editsync` or `Editsync`, since they are all Editsync extensions.

## Updating an extension

To update an extension, open a PR to [the `khulnasoft/extensions` repo](https://github.com/khulnasoft/extensions).

In your PR do the following:

1. Update the extension's submodule to the commit of the new version.
2. Update the `version` field for the extension in `extensions.toml`
   - Make sure the `version` matches the one set in `extension.toml` at the particular commit.

If you'd like to automate this process, there is a [community GitHub Action](https://github.com/huacnlee/editsync-extension-action) you can use.
