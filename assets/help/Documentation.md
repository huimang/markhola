# MarkHola Documentation

Current version: `v0.9.0`

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
- Export the current document to PDF or HTML, receive a successful-export prompt with an `Open` action, and print it
- Quit MarkHola with the main window's red close button while keeping tab-close commands document-scoped
- Install the architecture-specific Apple Silicon or Intel app for macOS 14.0 or later

## Menus

### File

- New
- Open
- Save
- Save As
- Print
- Export > PDF / HTML
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

## Notes

- Interface language changes apply immediately without reloading the current document.
- Markdown content, file names, and exported content are not translated.
- Operating-system dialogs continue to use the language provided by macOS.
- The Outline item is available for an opened document in readonly mode.
- Use `Command + 1` through `Command + 9` to select native tabs by their visible order.
- Follow System changes between Light and Dark when the macOS appearance changes.
- Rendered Markdown links do not use underlines.
- Light and Dark use separate visually comfortable code palettes.
- The default empty state does not show a status prompt; successful PDF and HTML exports show the output path and an `Open` action.
- MarkHola provides separately labeled Apple Silicon and Intel downloads for macOS 14.0 or later.
