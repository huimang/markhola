# MarkHola

This is another markdown reader and exporter software.Written by AI, free are the significal features for it.


## Basis Features

### v0.5.0
1. Readonly
   Open markdown file and can show the basic elements:
   - title, h1,h2,h3...
   - link
   - picture
   - table
2. Language: english only
3. Platform: Apple M4

### v0.6.0
1. Writable
   - You can edit the markdown file under this model.After you enter "Command + /", you can switch the model both for readonly and writeonly.
   - You can save the markdown file after edit it and enter the "Command + S"

### v0.6.1
1. Code Highlight Support
  - Display line number at the left side
  - Highlight the keyword by the type of the language
  - Display the language on the right top corner while the mouse hover the code area
2. Writable editor shortcut enhancement
  - Support "Command + A" to select all text while in writable mode
  - Support "Command + C", "Command + V", and "Command + X" for common text editing
  - Support "Command + Z" to undo the previous editor action
  - Support "Command + R" to redo the next editor action
  - Support "Ctrl + A" to move caret to the start of the current line
  - Support "Ctrl + E" to move caret to the end of the current line
  - Support "Tab" to insert 4 spaces and indent selected multiple lines
  - Support "Shift + Tab" to remove 4 spaces and outdent selected multiple lines
3. Writable editor line numbers
  - Display line numbers at the left side while editing Markdown

### v0.6.2
1. Mermaid diagram support
   - Render Mermaid fenced code blocks in readonly mode
   - Support common Mermaid diagram types such as flowchart, sequence, class, state, ER, gantt, pie, journey, git graph, mindmap, timeline, and sankey
   - Keep Mermaid source text editable in writable mode

### v0.6.3
1. Mainstream language syntax highlight enhancement
   - Improve readonly fenced code block highlighting coverage for mainstream languages
   - Ensure the languages already listed in `examples/languages.md` render with syntax highlighting when a language token is declared
   - Add language alias normalization for common fenced code block tokens such as `typescript`, `swift`, and `kotlin`
   - Keep unknown or untagged code blocks rendering safely as plain text fallback
2. Mathematical expression support
   - Support GitHub-style inline math expressions in readonly mode
   - Support GitHub-style block math expressions delimited by `$$...$$`
   - Support fenced `math` code blocks as display math in readonly mode
   - Keep raw math source editable in writable mode

### v0.7.0
1. Multiple Markdown documents in one app window
2. Theme directory for app shell styling

### v0.7.1
1. Support for Exporting PDF

### v0.7.2
1. Support for printing Markdown files
2. Save As for Markdown documents
3. Export Markdown documents to HTML
4. Built-in documentation entry in the Help menu
5. Bug fixes
   - Reload the current document correctly after using the page reload action

### v0.7.3
1. In-page find support in readonly mode
2. In-page find and replace support in writable mode
3. Code split. Reduce the single rs file size, split to three parts: interface,implements,tests.

### v0.7.4
1. Support creating blank Markdown documents
2. Save a new unsaved document by choosing a path first
3. Fix Mermaid and other async-rendered diagrams missing from the print output
4. Fix the case where print preview is visible but the actual printed paper is blank
5. Replace the app branding logo everywhere with `assets/logo.png`

### v0.7.5
1. Support `[toc]`
2. Enable to choose multiple markdown files while use "open file"
3. Bug fix: Drag and drop open should render the same document view as "Open File" (not a single `<pre>` node).
4. Theme selection persistence
   - After the user selects a theme, the app should remember that theme on the next launch
   - Add a small configuration persistence mechanism for current and future app preferences
   - The selected theme item in the native menu should show a check mark

### v0.7.6
1. Improve app shell theme support
   - Keep the current warm `default` theme
   - Add a clean `light` theme
   - Add a `dark` theme
2. Add a `View` menu before `Help`
   - Add `View > Theme`
   - Allow choosing `default`, `dark`, and `light`
   - Apply the selected theme immediately in the running app
3. Add a full-screen document view action in the `View` menu
   - Allow toggling fullscreen from the menu
   - Keep document reading and editing usable after entering fullscreen

### v0.7.7
1. Update the logo assets and unify the product branding UI
2. Continue to add more features into `v0.7.7`
3. Persist the selected app shell theme across app relaunches
4. Fix drag-and-drop opening for multiple Markdown files
   - Dragging 2 or more Markdown files into the app should open all files
   - Each dropped file should appear in its own tab
5. Refine the empty workspace UI
   - Reduce the bottom status bar height when no file is open
6. Move the footer status bar out of the WebView
   - Make the footer a native macOS window component
   - Keep the Markdown reading and editing area inside the WebView

### v0.7.8
1. Add document font-size controls under `View > Size`
   - Add `Zoom In`, `Zoom Out`, and `Reset` submenu actions
   - Keep the current document font sizes as the default `100%` setting
   - Apply size changes to both readonly preview and writable editor content
   - Remember the selected document size across app relaunches
2. Refine the workspace below the native macOS menu
   - Use a flatter document tab strip without a brand mark or file-type icons
   - Use one logo-derived accent color under the active tab
   - Define the shared Violet scale as MarkHola Violet `#6657E8`, Violet Mid `#ACA4F4`, and Violet Tint `#F2F0FF`
   - Define the shared Green scale as MarkHola Green `#17B890`, Green Mid `#81D9C3`, and Green Tint `#EAF9F5`
   - Use the shared brand palette for tabs, links, Outline, focus, hover, selection, and other app accent states
   - Use the shared brand palette for fenced-code surfaces, line numbers, language badges, and syntax highlighting
   - Use Green Tint as the Markdown table header background
   - Keep `+` fixed at the far right as the new-document shortcut and remove the unused more button
   - Show backward and forward tab navigation only when open tabs overflow
   - Add an Outline toggle that opens the current document TOC on the right
   - Remove the top document toolbar and consolidate read-only document information into the native bottom status bar in this order: path, Words, Lines, mode value, status value
   - Move the Outline toggle to the View menu and show a checkmark while the right-side Outline panel is visible
   - Keep line numbers in writable mode and omit a reading rail in readonly mode
   - Show the MarkHola app icon to the left of the centered empty-workspace message when no document is open
   - Number new blank-document titles sequentially as `Untitled-1`, `Untitled-2`, and so on
   - Remove failed Mermaid render artifacts so error SVGs cannot escape the document content area

### v0.8.0
1. Replace the WebView document tab strip with native macOS window tabs
   - Give each open Markdown document its own native window, WebView, footer, and document runtime
   - Group document windows with the native macOS window tab system
   - Support `Command + 1` through `Command + 9` for selecting tabs by their visible order
   - Support moving a tab into a separate window and merging compatible windows back into one tab group
   - Keep document titles, unsaved state, close confirmation, themes, font size, Outline, and footer information synchronized per window
   - Preserve correct open, save, close, drag-and-drop, and application-exit behavior across multiple windows

### v1.0.0
1. Two models: Readonly、writeonly.
2. Readonly:Default mode, you can open any markdown file.
   Support features:
   - Table
   - Link
   - Picture
   - Flow
   - Math Expression
   - Code Highlight
3. Writeonly:You can edit the markdown file under this model.After you enter "Command + /", you can switch the model both for readonly and writeonly.
4. Export to PDF. 
5. Support: English, Chinese
6. Platform: Apple Intel、Apple M、Windows 11

## Tech architecture
1. Written by Rust.
2. Embbed a chrominum browser for display markdown, support H5 labels.
