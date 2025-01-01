# Fonts

<!--
TBD: WIP. Editsync Fonts documentation. This is currently not linked from SUMMARY.md are so unpublished.
-->

Editsync ships two fonts: Editsync Plex Mono and Editsync Plex Sans. These are based on IBM Plex Mono and IBM Plex Sans, respectively.

<!--
TBD: Document how Editsync Plex font files were created. Repo links, etc.
-->

## Settings

<!--
TBD: Explain various font settings in Editsync.
-->

- Buffer fonts
  - `buffer-font-family`
  - `buffer-font-features`
  - `buffer-font-size`
  - `buffer-line-height`
- UI fonts
  - `ui_font_family`
  - `ui_font_fallbacks`
  - `ui_font_features`
  - `ui_font_weight`
  - `ui_font_size`
- Terminal fonts
  - `terminal.font-size`
  - `terminal.font-family`
  - `terminal.font-features`
- Other settings:
  - `active-pane-magnification`

## Old Editsync Fonts

Previously, Editsync shipped with `Editsync Mono` and `Editsync Sans`, customieditsync versions of the [Iosevka](https://typeof.net/Iosevka/) typeface. You can find more about them in the [editsync-fonts](https://github.com/khulnasoft/editsync-fonts/) repository.

Here's how you can use the old Editsync fonts instead of `Editsync Plex Mono` and `Editsync Plex Sans`:

1. Download [editsync-app-fonts-1.2.0.zip](https://github.com/khulnasoft/editsync-fonts/releases/download/1.2.0/editsync-app-fonts-1.2.0.zip) from the [editsync-fonts releases](https://github.com/khulnasoft/editsync-fonts/releases) page.
2. Open macOS `Font Book.app`
3. Unzip the file and drag the `ttf` files into the Font Book app.
4. Update your settings `ui_font_family` and `buffer_font_family` to use `Editsync Mono` or `Editsync Sans` in your `settings.json` file.

```json
{
  "ui_font_family": "Editsync Sans Extended",
  "buffer_font_family": "Editsync Mono Extend",
  "terminal": {
    "font-family": "Editsync Mono Extended"
  }
}
```

5. Note there will be red squiggles under the font name. (this is a bug, but harmless.)
