# Helm

Support for Helm in Editsync is provided by the community-maintained [Helm extension](https://github.com/cabrinha/helm.editsync).

- Tree-sitter: [tree-sitter-go-template](https://github.com/ngalaiko/tree-sitter-go-template/tree/master)
- Language Server: [mrjosh/helm-ls](https://github.com/mrjosh/helm-ls)

## Setup

Enable Helm language for Helm files by editing your `.editsync/settings.json` and adding:

```json
  "file_types": {
    "Helm": [
      "**/templates/**/*.tpl",
      "**/templates/**/*.yaml",
      "**/templates/**/*.yml",
      "**/helmfile.d/**/*.yaml",
      "**/helmfile.d/**/*.yml"
    ]
  }
```
