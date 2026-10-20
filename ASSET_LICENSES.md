# Graphics provenance and notices

The original source-code MIT notice in [LICENSE](LICENSE), including
Copyright (c) 2026 sattatrapy, is preserved. Source-code licensing is not a new
license grant over third-party game artwork.

## Original SattaTracker resource archive

Original item icons and pig/coin animation frames were imported byte-for-byte
from the resource archive distributed by `sattatrapy`:

- Source: https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip
- Release asset ID: 656244446; retrieved 2026-10-19.
- Archive size: 189,021,488 bytes.
- GitHub release SHA-256, independently verified:
  `83e55bb0e7d193e43a66fa2304f5eff867360fc10b51ee29b45d74ac708669ab`.
- Imported: 3,088 image files, 189,996,776 bytes, without re-encoding.
- `src-tauri/resources/icons/`: 2,845 original icon files.
- `src/assets/animations/pig/`: 121 original files, including unused spare
  pig_120.png; existing animation uses pig_0.png through pig_119.png.
- `src/assets/animations/coin/`: all 122 original coin frames.

The full source/destination mapping, actual MIME types, dimensions, original byte
hashes and decoded RGBA hashes are recorded in
[assets/original-assets.manifest.json](assets/original-assets.manifest.json).
Many original `.png` filenames actually contain WebP bytes; names and bytes are
preserved, and the application sends the actual MIME type.

The archive contains only image files and no LICENSE/NOTICE/README. No specific
redistribution restriction was found in the inspected archive or release notes;
this is not proof of an unrestricted license. Distribution by the original
project does not establish authorship of every image. Game artwork rights remain
with the respective game rightsholders; the decorative animation authorship is
not independently established. These resources are not claimed as Deanfinite's
original artwork, and this document does not relicense them under MIT. Preserve
any additional rightsholder notices when supplied. No author attribution has
been removed.

No resource archive updater runs in the app. The source URL above is provenance;
`tools/import-original-assets.py` is a developer-only, digest-pinned import tool.
No TLIDB/TITrack imagery or source code is downloaded by runtime or build.

## Graphics authored for Deanfinite

The geometric D design (`src/assets/local/app.svg`) and its raster/container
versions in `public/app-icon.png` and `src-tauri/icons/`, the inline attach/detach
SVG paths, and `src/assets/local/item.svg` are original additions for this fork,
distributed under the project's MIT license. The item SVG is an error-only
fallback, never the normal replacement for existing item icons or frames.
The D icon recipe is `tools/generate-app-icons.py`.

## Fonts and previous template resources

No font binaries are bundled. The app uses installed system fonts; the former
Google Inter/Material Symbols CSS links remain removed. Unused Vite/Vue logos
and inherited app icon variants remain removed. Existing user public_assets,
assets_version.json and archives are not read, migrated or deleted.
