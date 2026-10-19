# Graphics and fonts

The original code copyright notice in [LICENSE](LICENSE) remains unchanged.
The MIT license on source code does not establish rights to third-party game art.

## Graphics authored for Deanfinite

The following original geometric designs were created for this fork and are
distributed under the project's [MIT license](LICENSE):

- `src/assets/local/item.svg`: neutral item diamond, used for every item ID.
- `src/assets/local/coin.svg`: neutral geometric coin, replacing coin frames.
- `src/assets/local/effect.svg`: neutral decorative emblem, replacing pig frames.
- `src/assets/local/app.svg`: Deanfinite geometric D logo.
- `public/app-icon.png`, `src-tauri/icons/{32x32,128x128,128x128@2x,icon}.png`,
  `src-tauri/icons/icon.ico` and `icon.icns`: raster/container versions of the
  same original D design; recipe: `tools/generate-app-icons.py` (Pillow).
- The inline detach/attach SVG paths in `src/components/DraggableWidget.vue`.

No game images, downloaded artwork, third-party fonts or upstream animation
frames were used to produce these designs. Pillow is a development tool only;
it is not needed by the app, CI build or runtime.

## Removed or unbundled resources

| Previous resource | Provenance / distribution decision |
|---|---|
| Item PNGs and pig/coin frames from upstream `assets.zip` | Archive not downloaded for this change. Hosting by the original developer does not establish authorship or redistribution rights. Could include game artwork; rights remain unverified. Not copied, bundled or republished. |
| Previous Tauri-style app/tray/installer icons | Inherited repository files; exact artwork license not established here. Replaced with the original D design; unused Square/Store variants removed from source. |
| `public/vite.svg`, `src/assets/vue.svg` | Unused template logos removed. Their brand/trademark permissions are unnecessary for this build. |
| Google-hosted Inter and Material Symbols | External stylesheet links removed. No font bytes copied or bundled. Inter is generally published under SIL OFL and Material Symbols under Apache 2.0, but no exact downloaded font/version is redistributed here. |
| System fonts and Unicode/emoji | Requested from the user's installed OS fonts, not redistributed by Deanfinite. Appearance depends on the installed system. |

Existing user `public_assets`, `assets_version.json` and archive files are not
read, migrated or deleted by the new graphics code. This notice does not grant
rights over their contents. See [resource inventory and verification](docs/local-assets.md).
