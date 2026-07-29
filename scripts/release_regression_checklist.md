# Release Regression Checklist

Run automated regression first:

```bash
./scripts/release_regression.sh
```

Run this extended command when packaging a release candidate:

```bash
./scripts/release_regression.sh --with-package
```

Before publishing a GitHub release, complete the sandbox validation flow in:

```bash
scripts/release_publish_workflow.md
```

## Manual checks

1. Universal 2 candidate identity
   Expected: `scripts/verify_macos_architectures.sh --app dist/MarkHola.app --universal` passes.
   Expected: the main executable contains only `arm64` and `x86_64`.
   Expected: both slices and `LSMinimumSystemVersion` report macOS 14.0.
   Expected: the final assembled App signature is valid.

2. Empty launch close behavior
   Expected: launch the app with no document opened, press `Command+W`, and the app exits.

3. Basic Markdown rendering
   Open `examples/basic.md`.
   Expected: headings, links, blockquotes, tables, and images render normally.

4. New document regression
   Expected: `File > New` and `Command + N` both create a blank writable Markdown document.
   Expected: first save for a new unsaved document opens a save path chooser instead of failing.

5. Code highlight regression
   Open `examples/languages.md`.
   Expected: fenced blocks show line numbers and language badges.
   Expected: `typescript`, `swift`, and `kotlin` blocks are highlighted instead of plain fallback.

6. Mermaid regression
   Open `examples/mermaid.md`.
   Expected: Mermaid blocks render diagrams rather than remaining as plain code.

7. Math regression
   Open `examples/math.md`.
   Expected: inline math, `$$...$$`, and fenced `math` blocks render correctly.

8. PDF export regression
   Open `examples/pdf-export.md`.
   Expected: `File > Export > PDF` exports the current active tab only.
   Expected: the exported PDF keeps headings, table, code block, image, and math content.
   Expected: exporting from writable mode includes unsaved edits.

9. HTML export regression
   Open `examples/basic.md`.
   Expected: `File > Export > HTML` exports the current active tab as a standalone HTML file.
   Expected: the exported HTML keeps rendered Markdown styling and can load Mermaid and math enhancements.

10. Print regression
   Open `examples/basic.md` and `examples/mermaid.md`.
   Expected: `File > Print` and `Command + P` both open the system print panel for the current active tab.
   Expected: the print panel content reflects the current document instead of the application shell.
   Expected: writable-mode unsaved edits are included in the printed content.
   Expected: Mermaid flowcharts and other async-rendered diagrams appear in the print preview/output instead of staying blank.

11. Find regression
   Open `examples/basic.md`.
   Expected: `Command + F` and `Edit > Find` open the same find panel.
   Expected: readonly mode highlights matches, shows a stable match count, and supports `Enter`, `Shift + Enter`, `Next`, and `Previous`.
   Expected: writable mode can find, replace, and replace all within the current tab without breaking dirty state updates.

12. Documentation regression
   Expected: `Help > Documentation` opens the bundled release help markdown file inside the app.
   Expected: the document's `Current version` matches the version in `Cargo.toml`.
   Expected: its feature and menu descriptions include the target release changes and do not list removed behavior.

13. Multi-document regression
   Open `examples/basic.md` and `examples/multi-document.md`.
   Expected: tabs stay pinned at the top while document content scrolls.
   Expected: switching tabs preserves each document state.
   Expected: closing one of several tabs keeps the app open.
   Expected: closing the last opened document returns to the empty state instead of exiting.

14. Tab menu regression
   Expected: the `Tab` menu can switch tabs, close the current tab, close other tabs, and close all tabs.

15. Theme resource regression
   Open `examples/theme-showcase.md`.
   Expected: `View` appears before `Help`.
   Expected: `View > Theme` exposes `Follow System`, `Default`, and `Dark`.
   Expected: `Follow System` maps macOS Light to Default and macOS Dark to Dark.
   Expected: switching themes updates the running app immediately in readonly mode.
   Expected: switching themes updates the running app immediately in writable mode.
   Expected: packaged app contains `Contents/Resources/themes/default/layout.css`.
   Expected: packaged app contains `Contents/Resources/themes/dark/layout.css`.
   Expected: packaged app contains `Contents/Resources/help/Documentation.md`.
   Expected: each supported Help language file is present in `Contents/Resources/help`.

16. Inspect regression
   Right click in the preview area.
   Expected: the context menu still exposes `Inspect`.

17. Fullscreen regression
   Open `examples/theme-showcase.md`.
   Expected: `View > Toggle Full Screen` enters fullscreen.
   Expected: `View > Toggle Full Screen` exits fullscreen.
   Expected: the current document remains open and usable before, during, and after fullscreen.

## Pre-publish sandbox verification

Use the exact DMG candidate that will be uploaded to GitHub.

1. Mount the release DMG and launch the copied `MarkHola.app`
   Expected: the packaged app starts normally inside the sandboxed macOS environment.

2. Open a Markdown file through `File > Open`
   Expected: the packaged app can open a Markdown file without relying on local dev binaries.

3. Switch to writable mode, edit the file, and save it
   Expected: the window enters writable mode, accepts edits, and persists them to disk.

4. Return to readonly mode and verify rendered output
   Expected: the preview reflects the saved content from the packaged app.

5. Verify the target release feature in the packaged app
   Expected: the headline feature for the version works in the sandbox before the DMG is uploaded or the GitHub release is published.

6. Open `Help > Documentation` in every supported interface language in the packaged app
   Expected: each bundled document shows the target version and accurately describes the packaged feature and menu behavior.

7. Validate Apple Silicon runtime identity
   Expected: the exact copied candidate process path is recorded and startup/About report `aarch64`.

8. Validate Intel runtime identity
   Expected: a macOS 14+ Intel Mac runs the same DMG and startup/About report `x86_64`.
   Expected: if Product authorizes Rosetta instead, the exact candidate is forced to x86_64,
   `sysctl.proc_translated=1` is recorded, and the full accepted fallback behavior suite is run.

9. Freeze and re-read the release asset
   Expected: the final DMG SHA-256 is recorded before upload.
   Expected: the uploaded asset is downloaded again and has the same file name, size, and SHA-256.
   Expected: the release has one Universal 2 DMG rather than separate architecture assets.
