# MarkHola Documentation

Current version: `v0.9.2`

## What is MarkHola

MarkHola is a free Markdown reader and editor built with AI.

## Main Features

- Create, open, read, edit, and save local Markdown documents
- Switch between readonly and writable modes with `Command + /`
- Open multiple documents in native macOS tabs
- Switch the interface between English and Simplified Chinese
- Remember the selected language, theme, and document size across app relaunches
- Find text in readonly mode and find or replace text in writable mode
- Open a right-side document Outline in readonly mode
- Render Mermaid, including literal `\n` line breaks, math, tables, links, images, and highlighted code
- Export the current document to PNG, PDF, or HTML, receive a successful-export prompt with an `Open` action, and print it
- Export an absolute canonical Markdown source to PNG, PDF, or HTML with a one-shot application
  binary command that does not open the normal app workspace
- Quit MarkHola with the main window's red close button while keeping tab-close commands document-scoped
- Install the architecture-specific Apple Silicon or Intel app for macOS 14.0 or later

## Menus

### File

- New
- Open
- Save
- Save As
- Print
- Export > PNG / PDF / HTML
- Close
- Exit

### Edit

- Toggle Mode
- Undo / Redo
- Find
- Cut / Copy / Paste
- Select All

### Tab

- Next Tab
- Previous Tab
- Close Tab
- Close Other Tabs
- Close All Tabs

### View

- Theme > Follow System / Light / Dark
- Language > English / 简体中文
- Size > Zoom In / Zoom Out / Reset
- Outline
- Toggle Full Screen

### Help

- Documentation

## Offline CLI Export

Use the executable inside `MarkHola.app` for one command per process:

```bash
MARKHOLA_BIN="/absolute/path/to/MarkHola.app/Contents/MacOS/MarkHola"
"$MARKHOLA_BIN" export-png \
  --source=/absolute/input.md \
  --target=/absolute/output.png \
  --theme=light \
  --json
```

The public commands are `export-png`, `export-pdf`, `export-html`, `version`, and `help`.
`--source` and `--target` must use absolute, normalized, canonical paths. The default theme is
`light`; `--theme=dark` selects Dark. Existing output is preserved unless `--overwrite` is
provided. JSON responses use `schema_version: 1`.

Stable exit codes are:

- `0`: success
- `2`: invalid command, option, or schema
- `3`: invalid or unreadable source
- `4`: unsafe, unavailable, or conflicting target
- `5`: render or export failure
- `6`: resource limit or timeout
- `7`: internal or hidden-runtime failure

## Notes

- Interface language changes apply immediately without reloading the current document.
- Markdown content, file names, and exported content are not translated.
- Operating-system dialogs continue to use the language provided by macOS.
- The Outline item is available for an opened document in readonly mode.
- Use `Command + 1` through `Command + 9` to select native tabs by their visible order.
- Follow System changes between Light and Dark when the macOS appearance changes.
- Rendered Markdown links do not use underlines.
- Light and Dark use separate visually comfortable code palettes.
- The default empty state does not show a status prompt; successful PNG, PDF, and HTML exports show the output path and an `Open` action.
- MarkHola provides separately labeled Apple Silicon and Intel downloads for macOS 14.0 or later.
