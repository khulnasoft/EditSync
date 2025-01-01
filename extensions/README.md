# Editsync Extensions

This directory contains extensions for Editsync that are largely maintained by the Editsync team. They currently live in the Editsync repository for ease of maintenance.

If you are looking for the Editsync extension registry, see the [`khulnasoft/extensions`](https://github.com/khulnasoft/extensions) repo.

## Structure

Currently, Editsync includes support for a number of languages without requiring installing an extension. Those languages can be found under [`crates/languages/src`](https://github.com/khulnasoft/editsync/tree/main/crates/languages/src).

Support for all other languages is done via extensions. This directory ([extensions/](https://github.com/khulnasoft/editsync/tree/main/extensions/)) contains a number of officially maintained extensions. These extensions use the same [editsync_extension_api](https://docs.rs/editsync_extension_api/latest/editsync_extension_api/) available to all [Editsync Extensions](https://editsync.khulnasoft.com/extensions) for providing [language servers](https://editsync.khulnasoft.com/docs/extensions/languages#language-servers), [tree-sitter grammars](https://editsync.khulnasoft.com/docs/extensions/languages#grammar) and [tree-sitter queries](https://editsync.khulnasoft.com/docs/extensions/languages#tree-sitter-queries).

## Dev Extensions

See the docs for [Developing an Extension Locally](https://editsync.khulnasoft.com/docs/extensions/developing-extensions#developing-an-extension-locally) for how to work with one of these extensions.

## Updating

> [!NOTE]
> This update process is usually handled by Editsync staff.
> Community contributors should just submit a PR (step 1) and we'll take it from there.

The process for updating an extension in this directory has three parts.

1. Create a PR with your changes. (Merge it)
2. Bump the extension version in:

   - extensions/{language_name}/extension.toml
   - extensions/{language_name}/Cargo.toml
   - Cargo.lock

   You can do this manually, or with a script:

   ```sh
   # Output the current version for a given language
   ./script/language-extension-version <langname>

   # Update the version in `extension.toml` and `Cargo.toml` and trigger a `cargo check`
   ./script/language-extension-version <langname> <new_version>
   ```

   Commit your changes to a branch, push a PR and merge it.

3. Open a PR to [`khulnasoft/extensions`](https://github.com/khulnasoft/extensions) repo that updates the extension in question

Edit [`extensions.toml`](https://github.com/khulnasoft/extensions/blob/main/extensions.toml) in the extensions repo to reflect the new version you set above and update the submodule latest Editsync commit.

```sh
# Go into your clone of the extensions repo
cd ../extensions

# Update
git checkout main
git pull
just init-submodule extensions/editsync

# Update the Editsync submodule
cd extensions/editsync
git checkout main
git pull
cd -
git add extensions.toml extensions/editsync
```
