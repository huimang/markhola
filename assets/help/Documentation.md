# MarkHola Documentation

![Application logo](../logo.png)

Current version: `v0.8.0`

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
- Render Mermaid, math, tables, links, images, and highlighted code
- Export the current document to PDF or HTML and print it

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
- Tab 1 through Tab 9 (`Command + 1` through `Command + 9`)
- Close Tab
- Close Other Tabs
- Close All Tabs

### View

- Theme > Default / Dark / Light
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
