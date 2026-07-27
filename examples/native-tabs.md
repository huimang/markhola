# Native macOS Tabs

This document verifies the native tab behavior in MarkHola v0.8.1.

## Open another tab

Open <multi-document.md> and confirm that it appears in the same native macOS tab group.

Opening the link again should activate the existing native tab instead of creating a duplicate.

## Switch by number

Use `Command + 1` and `Command + 2` to switch between the visible native tabs.

The document title, content, Outline, and footer should always describe the selected tab.

After opening multiple files together, enlarge the window from the title bar and switch between
tabs. The file path should stay on the left while Words and Lines remain aligned to the right edge.

Double-click the title bar to enlarge the window, switch to another tab, and double-click the title
bar again. The window should return to the size it had before it was enlarged.

## Tab menu

Open the `Tab` menu and confirm it does not list repeated `Tab 1` through `Tab 9` items.

Previous, next, and close actions should remain available in the menu.

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
