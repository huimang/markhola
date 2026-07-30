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

### v0.7.9
1. Add English and Simplified Chinese interface languages
   - Use English as the default language when no preference has been saved
   - Add `View > Language > English / 简体中文`
   - Apply language changes immediately to MarkHola's native menus, WebView interface, status messages, alerts, and bundled Help
   - Remember the selected language across app relaunches
   - Keep Markdown document content unchanged and leave operating-system-owned dialogs in the system language
2. Match native tab text-selection behavior
   - Prevent document tab titles from being selected by pointer dragging
   - Preserve tab activation, close buttons, keyboard focus, overflow navigation, and accessibility behavior
3. Fix and simplify the native View menu
   - Show `Outline` as the visible menu-item title instead of leaving the row blank
   - Keep the existing enabled, disabled, checked, unchecked, and toggle behavior unchanged
   - Verify the visible label in both the empty workspace and an opened readonly document
   - Remove the unused macOS `Show Tab Bar` and `Show All Tabs` items while MarkHola still uses its WebView document tabs
   - Keep MarkHola's own `Tab` menu and WebView tab controls unchanged
   - Remove the duplicate `Toggle Mode` item from the File menu while keeping it in Edit and
     preserving `Command + /`
4. Keep the bundled Help documentation synchronized with every release
   - Update the English and Simplified Chinese bundled Help files to the target version before packaging
   - Open the Help file matching the selected interface language
   - Keep both files' user-visible feature and menu descriptions aligned with the packaged app
   - Block release validation when either bundled Help version does not match the package version
5. Reuse tabs when opening local Markdown links
   - If a clicked local Markdown link targets a file that is already open, activate the existing tab instead of opening a duplicate
   - If the target file is not open, open it in a new tab and make it the active tab
   - Keep non-Markdown links and in-document anchor links on their existing behavior
6. Add angle-bracket shorthand links for Markdown text
   - Support `<xx>` as shorthand for `[xx](xx)` while rendering Markdown
   - Keep standard Markdown links, autolinks, and inline code behavior unchanged
   - Preserve the original Markdown source text in writable mode

### v0.8.0
1. Replace the current document tab strip with native macOS window tabs
   - Use native macOS tabs to replace the current custom document tab UI
   - Keep the existing single-window document workspace model as much as possible instead of expanding to multi-window document runtime changes in this version
   - Support `Command + 1` through `Command + 9` for switching tabs by their visible order
   - Show the available `⌘1` through `⌘9` shortcut on the right side of each corresponding native tab
   - Keep open, save, close, unsaved-state, theme, font-size, Outline, and footer behavior working after the tab replacement
   - Extend both brand scales with Violet Subtle `#F9F8FF` and Green Subtle `#FAFEFD`
   - Add a third shared Gray scale for neutral application surfaces alongside the Violet and Green scales
   - Use a subtle Gray Tint title-bar material, Green Tint for native tabs, and Green Subtle for the document background so the three surfaces remain visually distinct
   - Reduce the native bottom status bar height to 34 px while keeping its labels vertically aligned
   - Use the Gray scale for the native footer, keep the path on the left, right-align Words and Lines, remove the mode and status columns, and keep all labels on the same lower-brightness foreground color
   - Increase the centered document reading width from 860 px to 960 px across all themes
   - Render Markdown tables with straight corners across all themes

### v0.8.1
1. Clean up the native Tab menu
   - Remove the repeated `Tab 1` through `Tab 9` menu items
   - Keep `Command + 1` through `Command + 9` switching available through the native Tab shortcuts
   - Keep previous, next, close current, close other, and close all Tab menu actions unchanged
2. Remove underlines from rendered Markdown links
   - Keep links mapped to the shared Violet and Green theme colors
   - Do not show an underline in default, visited, hover, focus, or active states
   - Apply the same link treatment to Default and Dark themes and exported output
3. Simplify themes and follow the macOS appearance
   - Remove the separate Light theme
   - Offer Follow System, Default, and Dark choices in the Theme menu
   - Map the macOS Light appearance to MarkHola Default and the macOS Dark appearance to MarkHola Dark while Follow System is selected
   - Apply macOS appearance changes immediately to every native Tab surface without restarting the app
   - Keep manually selected Default or Dark fixed until the user selects Follow System again
   - Migrate the removed stored Light preference to Default
4. Improve code-block visual comfort
   - Use a pale gray-violet code surface in Default instead of a large dark block
   - Use a soft dark gray-violet code surface in Dark instead of pure black
   - Keep 14 px code text near `6:1–8:1`, line numbers near `4.5:1–5.5:1`, separators at least `3:1`, and language labels at least `4.5:1`
   - Use separate low-saturation syntax palettes for Default and Dark
   - Reduce keyword weight and avoid combining excessive brightness, saturation, and bold emphasis
5. Keep native footer metadata aligned after native Tab window changes
   - Relayout the active native footer whenever a document surface becomes active
   - Keep Words and Lines right-aligned after opening multiple files, enlarging the window, and switching tabs
   - Keep the file path on the left and preserve the compact footer height
6. Keep native Tab window zoom state consistent
   - Synchronize the active window's zoomed or restored state when switching native tabs
   - Preserve the original window size so double-clicking the title bar can restore it after a tab switch
### v0.8.2
1. Non-visual items already implemented
   - Keep native tab closing separate from application closing, while making the main window close
     button quit MarkHola
   - Keep footer and editor source line counts aligned
   - Render Mermaid content so literal `\n` sequences appear as visible line breaks in app
     preview, HTML export, and PDF/print output
   - Keep the default empty workspace silent instead of showing the bottom-right default prompt
   - Show visible success feedback after exporting PDF or HTML, including the output path and an
     `Open` action
   - Keep export-related error feedback visible as a separate error-only status path
2. Keep visual editing deferred
   - Do not bring back Typora-style inline mixed reading and editing in `v0.8.2`
   - Do not bring back localized mixed visual editing for complex rendered blocks in `v0.8.2`
   - Do not bring back broader incremental mixed-view editing updates in `v0.8.2`

### v0.9.0
1. Add Apple Intel support
   - Run the macOS app on Apple Intel in addition to Apple Silicon
   - Keep the existing macOS document, export, menu, and packaging behavior aligned across both chip families
   - Publish two explicitly labeled DMG assets instead of one unlabeled Universal 2 download:
     - `MarkHola-0.9.0-apple-silicon.dmg`, containing an `arm64` application
     - `MarkHola-0.9.0-intel.dmg`, containing an `x86_64` application
   - Do not publish the existing unlabeled Universal 2 DMG as a v0.9.0 release asset
   - Validate each exact architecture-specific DMG independently before publication
   - Allow the Intel DMG to be validated either on a physical Intel Mac or in an x86_64 macOS 14+
     virtual or fully emulated guest
   - When a virtual or emulated guest is used, require proof that the guest and MarkHola process are
     `x86_64`, that `sysctl.proc_translated=0`, and that the same Intel acceptance matrix passes
   - Do not treat an arm64 macOS guest, Tart on Apple Silicon, or Rosetta translation as satisfying
     the Intel gate
   - If physical Intel hardware is not used, disclose the remaining Intel CPU/GPU and native window
     system risk in the release notes

### v0.9.1
1. Export the current document as a PNG image
   - Add a user-visible export action for saving the currently active Markdown document as PNG
   - Export the complete rendered document rather than only the visible viewport
   - Preserve the same rendered Markdown, theme, Mermaid, math, and local-image behavior used by
     the current document preview
   - Let the caller choose the output path and report the final path or a clear failure reason
   - Keep PNG export separate from normal Markdown save behavior while sharing the established
     rendered-document export pipeline with PDF and HTML
2. Add a local process-command protocol for export and save automation
   - Allow a local automation client to send structured commands to a running MarkHola process
     without taking keyboard focus or driving the visible UI
   - Support exporting the active document as PNG, PDF, or standalone HTML through explicit
     `export_png`, `export_pdf`, and `export_html` commands
   - Support saving the active writable document to its current path and saving the current
     in-memory document to another path through explicit `save_document` and `save_document_as`
     commands
   - Make protocol PDF and HTML export reuse the same production services as the existing
     user-visible PDF and HTML actions, including current in-memory content, rendered Mermaid,
     math, local images, styling, success paths, and stable failures
   - Make protocol Save As reuse the production Save As service without showing a save panel, so
     validation can write to an isolated temporary file instead of overwriting the source fixture
   - Keep the original source file unchanged after Save As, switch the active document to the new
     canonical path on success, and return the new document identity and path
   - Require each request to carry a request ID, command name, target document identity when needed,
     output path or save options, and an explicit overwrite policy
   - Return a structured response with the request ID, success or failure status, affected document,
     final output path when applicable, and a stable error code and message
   - Keep command execution bound to the intended running MarkHola instance and fail closed when the
     target document is missing, ambiguous, readonly, or no longer current
   - Keep the protocol local-only and avoid exposing a network-listening service
   - Reuse the same application save and export paths as user-triggered actions so automated
     validation proves production behavior instead of a separate test-only implementation
3. Add validation coverage for UI and non-UI export and save operations
   - Verify PNG, PDF, and HTML export from the visible application actions and through the local
     process-command protocol against the exact packaged candidate
   - Verify `save_document` and `save_document_as` through the protocol, including current-path
     saves, isolated Save As targets, source preservation, and reopen/content identity
   - Verify deterministic request/response correlation, output-file identity, overwrite handling,
     Save As isolation from the source fixture, failure responses, and no-focus-stealing behavior
   - Compare UI-triggered and protocol-triggered PNG, PDF, and HTML outputs at the production-service
     boundary so the protocol cannot silently use a test-only export implementation
   - Use the protocol to accelerate release validation without treating protocol success alone as a
     substitute for visual inspection of representative PNG, PDF, and HTML output
4. Rename the user-visible Default theme to Light
   - Show `Light` instead of `Default` in the Theme menu, Help, README, examples, status text, and
     other user-visible surfaces
   - Preserve the internal and persisted `default` theme key, existing CSS/resource paths, and
     compatibility with stored `default` or legacy `light` preferences
   - Keep the Light theme's colors and rendering behavior unchanged; this is a naming improvement,
     not a new palette
   - When protocol responses expose theme state, return the stable canonical key `default` and the
     user-visible display name `Light` as separate fields
   - Verify Follow System maps macOS Light to the renamed Light theme, manual Light/Dark selection
     remains stable, and UI/PNG/PDF/HTML output keeps the existing theme behavior

### v0.9.2
Start this milestone after v0.9.1 is successfully published.

1. Add one-shot command-line document export through the MarkHola application binary
   - Support `export-png`, `export-pdf`, and `export-html` commands without opening a visible
     MarkHola window, taking keyboard focus, or showing native file panels
   - Accept an explicit Markdown source and output target, for example:
     `markhola export-pdf --source=/Users/ronnie/a.md --target=/tmp/a.pdf`
   - Reuse the same production rendering and export services as the visible application and the
     v0.9.1 local process-command protocol instead of introducing a CLI-only renderer
   - Preserve Markdown, theme, syntax highlighting, Mermaid, math, and local-image behavior, with
     relative local assets resolved from the source document directory
   - Allow the implementation to initialize non-visible AppKit or WebKit infrastructure when
     required for rendering, but do not create visible windows or remain running after completion
2. Define a safe and automation-friendly CLI contract
   - Require a canonical, absolute, readable `.md` source path and a format-correct absolute target
     path whose canonical parent already exists and is writable
   - Preserve the source file byte-for-byte and reject directories, application bundles, symlinks,
     unsafe path changes, and existing targets unless `--overwrite` is explicitly supplied
   - Write output atomically, clean temporary files on failure or cancellation, and return only
     after the final output has been committed or a stable failure has been reported
   - Support `help` and `version`, concise human-readable output, optional machine-readable JSON,
     and documented stable process exit codes
   - Execute exactly one command per process and do not start the v0.9.1 socket server or a
     long-running application event loop
3. Keep the first CLI release intentionally bounded
   - Do not add document editing, save, Save As, running-instance control, arbitrary filesystem
     access, shell execution, WebView JavaScript, stdin streaming, batch, watch mode, or global
     command installation in v0.9.2
   - Keep running-instance automation in the v0.9.1 Unix-domain-socket protocol and offline
     source-to-artifact conversion in the v0.9.2 CLI
   - Add paired technical design and test documents before implementation and provide a tracked
     example Markdown file that validates CLI PNG, PDF, and HTML output
   - Verify no visible window or focus change, exact source and output identity, source
     preservation, overwrite and path safety, stable exit codes and JSON responses, representative
     visual output, and parity with the shared production export services

### v1.0.0
Start this milestone immediately after v0.9.2 is successfully published.

1. Detect external Markdown file changes while a document is open
   - If the current document has no unsaved changes, automatically reload the latest on-disk content
   - If the current document has unsaved changes, prompt the user to choose one of:
     - Reload the latest on-disk content directly
     - Save the current in-memory content to a temporary file, then load the latest on-disk content
     - Keep the current document unchanged
   - Store conflict-recovery copies under the Apple-provided application temporary directory, not
     inside the installed or signed application bundle
   - Show the exact temporary-copy path to the user and do not present it as a durable backup
   - Avoid repeating the same prompt until the on-disk file changes again
   - When saving, compare the on-disk file content or file version against the version originally loaded for editing
   - If the file changed on disk during editing, prompt the user to choose one of:
     - Overwrite the current on-disk file with the editor content
     - Save the current editor content to a temporary file, then load the latest on-disk content
     - Keep the current document unchanged
2. Add large-view support for images and Mermaid diagrams
   - Allow image and Mermaid blocks to be opened in an overlay when the normal document layout is
     too small
   - Support zooming and panning inside the overlay
   - Make large images and diagrams readable without requiring permanent document font-size changes
   - Keep the original document content unchanged while entering and leaving the larger viewing mode
   - Keep this capability separate from document font-size changes and document-view zoom
   - Treat this as a block-focused viewing mode for oversized rendered content rather than a global document scaling action
3. Add a dedicated Settings page for MarkHola configuration
   - Provide one configuration entry for the user to manage app settings instead of scattering them only across menus
   - Cover theme, document font size, and interface language in the first version
   - Support `Command +` and `Command -` to increase or decrease document font size
   - Treat document font size as a typography setting for the current reading and editing experience
   - Let font-size changes affect text-centric layout such as paragraphs, headings, lists,
     blockquotes, code blocks, and spacing
   - Allow the visible document reading area to respond with proportional layout adjustments, but
     do not treat font-size changes as full document zoom
   - Keep the page style consistent with MarkHola's existing visual language
   - Allow the Settings page to become the shared entry point for future configuration items

4. Align the 1.0 release documentation and validation surface
   - Update `README.md`, bundled Help, examples, validation records, and release workflow before
     publishing v1.0.0
   - Keep the existing Apple Intel and Apple Silicon support as a compatibility baseline rather than
     treating it as new v1.0.0 feature scope

### v1.0.2
1. Add document-view zoom for reading
   - Add macOS two-finger pinch zoom for document viewing
   - Use system trackpad pinch gestures to zoom the document view itself instead of changing the saved document font-size preference
   - Treat document-view zoom as a viewport-scale behavior that can enlarge rendered content such as images, tables, formulas, and diagrams together
   - Keep document-view zoom separate from the saved document font-size setting

### Post-v1.5.0 (version TBD)
1. Add Windows 11 support
   - Run MarkHola on Windows 11 with the same core Markdown reading flow as macOS
   - Keep readonly and writable document behavior aligned with the existing product scope
   - Preserve the currently supported Markdown rendering set on Windows as far as the chosen WebView stack allows
   - Support opening local Markdown files, editing, saving, and basic document navigation on Windows
   - Provide Windows-specific packaging and installer output instead of reusing macOS-only release artifacts
   - Exclude Apple-only packaging resources and build steps from the Windows release flow
2. Add platform-aware packaging slimming after Windows support exists
   - Split platform-specific build, packaging, and release dependencies so each target platform only ships what it actually needs
   - Remove Apple-only packaging assets, libraries, and release steps from Windows artifacts
   - Prevent Windows-only packaging assets, libraries, and release steps from being pulled into macOS artifacts
   - Keep common Markdown, rendering, and document logic shared while isolating platform-exclusive runtime and packaging code
   - Reduce installer and application bundle size by target-aware dependency selection, conditional compilation, and packaging-resource pruning

### v1.1.0
1. Add MarkHola CLI capabilities for AI-facing automation
   - Provide a stable `markhola` command-line entry for opening Markdown files from external tools
   - Support opening a local Markdown file by explicit path
   - Support opening generated Markdown text as a temporary document with a caller-provided title
   - Define clear exit codes, stdout/stderr behavior, and failure messages so external agents can call the CLI reliably
   - Keep the CLI scope focused on safe document-opening actions first, without requiring in-app AI UI

### v1.1.1
1. Add MCP support for MarkHola operations
   - Wrap the stable CLI operations as MCP tools instead of requiring agents to assemble shell commands directly
   - Expose at least document-opening tools for local files and generated temporary Markdown content
   - Keep the MCP layer thin so MarkHola's app behavior stays owned by the existing application runtime
   - Reuse the same document-opening path validation and tab-activation behavior as the native app flow

### v1.1.2
1. Publish Tool schemas for AI discovery
   - Define structured tool names, descriptions, argument schemas, return fields, and side-effect expectations for the MCP tools
   - Document which tools are read-only, which tools open or focus app windows, and which tools may create temporary files
   - Keep the schema versioned so future AI-facing operations can expand without breaking existing integrations

### v1.1.3
1. Make MarkHola's AI-facing operations discoverable and usable by external AI systems
   - Ensure AI clients can understand the available MarkHola operations from the published Tool schemas
   - Validate the end-to-end path `CLI support -> MCP support -> Tool schemas -> AI understands available operations`
   - Use the resulting foundation for future AI workflows such as Confluence-to-Markdown import, external content ingestion, and in-app AI entry points
   - Keep direct AI chat UI as a later optional product layer on top of these integration capabilities

## Technical architecture reference

See [`docs/technical-architecture.md`](docs/technical-architecture.md) for the
current tracked architecture and technology-stack description.
