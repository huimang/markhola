# Workspace Visual Verification

Use this document to verify the `v0.7.9` localized interface, tab strip, and right-side Outline panel.

## Local markdown links

Use these links to verify local Markdown routing inside the current workspace:

- [Reuse the existing languages tab](languages.md)
- [Open the mermaid example in a tab](mermaid.md)
- [Jump to the writable layout section in this file](#writable-layout)

## Tab strip

Open this file together with several other files from `examples/`.

Confirm that:

- tabs contain file names without Markdown icons
- dragging across a tab title does not select its text
- document text remains selectable
- the active tab uses one logo-derived underline color
- the `+` button creates a blank document
- backward and forward controls appear only when tabs overflow

## Outline panel

Open the Outline control in readonly mode.

Confirm that `View > Outline` is visible in English and `视图 > 大纲` is visible in Simplified
Chinese. The View menu should not contain `Show Tab Bar` or `Show All Tabs`.

### Heading order

The Outline should list headings in the same order as this document.

### Heading nesting

This level-three heading should appear nested below `Outline panel`.

#### Deeper heading

This level-four heading verifies deeper visual nesting.

## Readonly layout

Readonly mode should show the Markdown document without a left reading rail or line-number column.

## Writable layout

Switch to writable mode with `Command + /`.

The existing editor line numbers should appear and remain aligned while typing and scrolling.

### Size compatibility

Use `View > Size > Zoom In`, `Zoom Out`, and `Reset`.

Editor text and line numbers should resize together without changing the native footer.

## Theme compatibility

Verify the workspace with:

1. Default
2. Dark
3. Follow System

Tab structure and Outline behavior should remain consistent in every theme.

## Language compatibility

Use `View > Language` to switch between English and Simplified Chinese.

Confirm that:

- menus, empty state, Find, Outline, About, status text, and Help change immediately
- the current document, active tab, unsaved edits, and Markdown content do not change
- the selected language remains active after relaunching MarkHola
