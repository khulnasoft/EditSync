# Diff

Diff support is available natively in Editsync.

- Tree Sitter: [khulnasoft/the-mikedavis/tree-sitter-diff](https://github.com/the-mikedavis/tree-sitter-diff)

## Configuration

Editsync will not attempt to format diff files and has [`remove_trailing_whitespace_on_save`](https://editsync.khulnasoft.com/docs/configuring-editsync#remove-trailing-whitespace-on-save) and [`ensure-final-newline-on-save`](https://editsync.khulnasoft.com/docs/configuring-editsync#ensure-final-newline-on-save) set to false.

Editsync will automatically recognize files with `patch` and `diff` extensions as Diff files. To recognize other extensions, add them to `file_types` in your Editsync settings.json:

```json
  "file_types": {
    "Diff": ["dif"]
  },
```
