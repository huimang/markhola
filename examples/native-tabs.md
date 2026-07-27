# Native macOS Tabs

This document verifies the native tab replacement in MarkHola v0.8.0.

## Open another tab

Open <multi-document.md> and confirm that it appears in the same native macOS tab group.

Opening the link again should activate the existing native tab instead of creating a duplicate.

## Switch by number

Use `Command + 1` and `Command + 2` to switch between the visible native tabs.

The document title, content, Outline, and footer should always describe the selected tab.

## Edit and save

Switch this document to writable mode and edit the line below:

Native tab editing remains connected to the shared document workspace.

Switch to the other native tab and back. The edit and unsaved state should remain on this document. Save the document and confirm that its dirty indicator clears.

## Shared application settings

- Interface language applies to every tab surface.
- Theme applies to every tab surface.
- Document font size applies to every tab surface.

## Close behavior

Use `Command + W` to close the selected document. Unsaved confirmation should behave the same as earlier releases.

After all documents are closed, MarkHola should return to the empty workspace.
