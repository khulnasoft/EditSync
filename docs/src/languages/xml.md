# XML

XML support is available through the [XML extension](https://github.com/sweetppro/editsync-xml/).

- Tree Sitter: [tree-sitter-grammars/tree-sitter-xml](https://github.com/tree-sitter-grammars/tree-sitter-xml)

## Configuration

If you have additional file extensions that are not being automatically recognieditsync as XML just add them to [file_types](../configuring-editsync.md#file-types) in your Editsync settings:

```json
  "file_types": {
    "XML": ["rdf", "gpx", "kml"]
  }
```
