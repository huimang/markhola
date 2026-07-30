# MarkHola

![MarkHola logo](assets/logo.png)

MarkHola is a free Markdown reader and editor built with AI.

## Current Version

- `v0.9.0`

## Features

- Readonly and writable modes with `Command + /` mode switching
- Create a new blank Markdown document with `File > New` or `Command + N`
- Open local `.md` and `.markdown` files (Open File supports multi-select)
- Open and keep multiple Markdown documents in one native macOS window tab group
- Switch native tabs by their visible order with `Command + 1` through `Command + 9`
- Show `⌘1` through `⌘9` on the right side of the corresponding native tabs
- Drag native tabs to reorder them while keeping numbered switching aligned with the new order
- Switch the interface between English and Simplified Chinese from `View > Language`
- Reopen the app with the previously selected interface language still active
- Open a heading outline panel from the checked `View > Outline` menu item in readonly mode
- Use the compact read-only native bottom status bar with the file path on the left and right-aligned Words and Lines in one consistent neutral text color
- Export the full rendered current document to PNG from `File > Export > PNG`
- Export the current document to PDF from `File > Export > PDF`
- Export the current document to HTML from `File > Export > HTML`
- Show the exported path and an `Open` action after a successful PDF or HTML export while keeping the default empty state silent
- Print the current document from `File > Print`
- Switch app shell themes from `View > Theme > Follow System / Light / Dark`
- Follow the macOS Light or Dark appearance automatically unless a fixed theme is selected
- Use Green Subtle `#FAFEFD` instead of pale yellow for the Light theme background
- Use a subtle Gray Tint title-bar material, Green Tint `#EAF9F5` for native tabs, and Green Subtle `#FAFEFD` for the document surface
- Use a wider centered 960 px reading area across all themes
- Render Markdown tables with straight corners
- Reopen the app with the previously selected shell theme still active
- Use shared Violet, Green, and Gray color scales across native tabs, document surfaces, the native footer, links, Outline, tables, and fenced code highlighting
- Adjust document text from `View > Size > Zoom In / Zoom Out / Reset`
- Reopen the app with the previously selected document size still active
- Load the app shell themes from editable files under `themes/<theme>/layout.css`
- Save the current document with `Command + S`
- Save a new unsaved document by choosing a path on first save
- Save the current document to another path with `File > Save As`
- Open the bundled documentation from `Help > Documentation`
- Render headings, links, images, tables, lists, blockquotes, and code blocks
- Render Markdown links without underlines while retaining themed hover and focus colors
- Render angle-bracket shorthand links such as `<README.md>` as clickable links in readonly mode
- Syntax-highlighted fenced code blocks in readonly mode
- Improved mainstream language highlight coverage for fenced code blocks
- Mathematical expressions in readonly mode, including inline math, `$$...$$`, and fenced `math` blocks
- Code block line numbers and hover language badges in readonly mode
- Use separate low-stimulation code palettes for Light and Dark
- Render Mermaid fenced code blocks in readonly mode
- Render literal `\n` in Mermaid node and edge labels as visible line breaks
- Support `[toc]` placeholder for table of contents in readonly mode
- In-page find in readonly mode with `Command + F`
- In-page find and replace in writable mode with `Command + F`
- Writable editor line numbers
- Writable editor shortcuts:
  - `Command + A` select all
  - `Command + C / V / X` copy, paste, and cut
  - `Command + Z / R` undo and redo
  - `Ctrl + A / E` move to line start and line end
  - `Tab / Shift + Tab` indent and outdent, including multi-line selections
- `Command + W` close the current native document tab
- Quit MarkHola from the main window's red close button while keeping tab-close commands document-scoped
- Drag and drop one or more Markdown files into the window
- Toggle fullscreen document viewing from `View > Toggle Full Screen`
- Open Markdown files from Finder on macOS
- Open external links in the default browser
- macOS app bundle and DMG packaging
- Separate architecture-specific macOS DMGs for Apple Silicon and Intel

## Platform

- macOS 14.0 or later on Apple Silicon
- macOS 14.0 or later on Apple Intel

## Tech Stack

- Rust

## Third-Party Libraries

- `block2`
- `chardetng`
- `encoding_rs`
- `icns`
- `lopdf`
- `objc2`
- `objc2-app-kit`
- `objc2-core-foundation`
- `objc2-foundation`
- `objc2-web-kit`
- `open`
- `pulldown-cmark`
- `rfd`
- `serde`
- `serde_json`
- `syntect`
- `tao`
- `url`
- `wry`
- `yaml-rust`

## Development

Run tests:

```bash
cargo test
```

Run release regression checks:

```bash
./scripts/release_regression.sh
```

Run release regression checks with packaging:

```bash
./scripts/release_regression.sh --with-package
```

Use the full release publish workflow before uploading a GitHub release:

```bash
open scripts/release_publish_workflow.md
```

Build the app:

```bash
cargo build
```

Create the macOS app bundle and DMG:

```bash
./scripts/package_dmg.sh
```

The macOS build scripts require `rustup` with the fixed Rust `1.95.0` toolchain and both
`aarch64-apple-darwin` and `x86_64-apple-darwin` targets declared by `rust-toolchain.toml`.
The build preflight reports the exact installation command when any prerequisite is missing.

Create a fast host-architecture app for local development:

```bash
./scripts/build_app.sh
```

Create and verify an Apple Silicon app without packaging:

```bash
./scripts/build_app.sh \
  --target aarch64-apple-darwin \
  --app dist/MarkHola-apple-silicon.app
```

Create and verify an Intel app without packaging:

```bash
./scripts/build_app.sh \
  --target x86_64-apple-darwin \
  --app dist/MarkHola-intel.app
```

Run the standalone thin-architecture gate tests:

```bash
./scripts/test_verify_macos_architectures.sh
```

Release order for GitHub publishing:

1. Engineering runs `./scripts/release_regression.sh --with-package` from one clean commit.
2. Testing validates each exact architecture-specific DMG in its matching accepted environment.
3. Product uploads the already-validated pair to the GitHub release draft.
4. Product publishes only after both architecture gates pass.

## Project Structure

- `src/`: desktop app source code
- `docs/technical-architecture.md`: current technical stack and architecture boundaries
- `src/bin/make_icns.rs`: macOS icon generation helper
- `assets/`: logo and icon sources
- `examples/`: sample Markdown files for manual verification
- `scripts/`: packaging scripts
- `scripts/release_publish_workflow.md`: pre-publish sandbox validation and GitHub release workflow
- `themes/`: directly editable app theme files
- `assets/help/`: bundled in-app help markdown files

## GitHub

<https://github.com/huimang/markhola>
