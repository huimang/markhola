# Workspace Visual Verification

Use this document to verify the `v0.7.8` tab strip and right-side Outline panel.

## Tab strip

Open this file together with several other files from `examples/`.

Confirm that:

- tabs contain file names without Markdown icons
- the active tab uses one logo-derived underline color
- the `+` button creates a blank document
- backward and forward controls appear only when tabs overflow

## Outline panel

Open the Outline control in readonly mode.

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
3. Light

Tab structure and Outline behavior should remain consistent in every theme.
