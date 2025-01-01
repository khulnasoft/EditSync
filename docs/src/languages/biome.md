# Biome

[Biome](https://biomejs.dev/) support in Editsync is provided by the community-maintained [Biome extension](https://github.com/biomejs/biome-editsync).
Report issues to: [https://github.com/biomejs/biome-editsync/issues](https://github.com/biomejs/biome-editsync/issues)

- Language Server: [biomejs/biome](https://github.com/biomejs/biome)

## Biome Language Support

The Biome extension includes support for the following languages:

- JavaScript
- TypeScript
- JSX
- TSX
- JSON
- JSONC
- Vue.js
- Astro
- Svelte
- CSS

## Configuration

By default, the `biome.json` file is required to be in the root of the workspace.

```json
{
  "$schema": "https://biomejs.dev/schemas/1.8.3/schema.json"
}
```

For a full list of `biome.json` options see [Biome Configuration](https://biomejs.dev/reference/configuration/) documentation.

See the [Biome Editsync Extension README](https://github.com/biomejs/biome-editsync) for a complete list of features and configuration options.
