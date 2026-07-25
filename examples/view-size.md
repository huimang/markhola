# View Size Showcase

Use this file to verify `View > Size > Zoom In / Zoom Out / Reset` in `v0.7.8`.

## Readable paragraph

The default `100%` setting should look exactly like the previous MarkHola document typography. Increase and decrease the size to compare body text, **bold text**, *italic text*, and [links](https://example.com).

### Smaller heading

Heading proportions should remain consistent while the document font size changes.

> Blockquotes should resize with the surrounding Markdown content.

## Lists

1. Zoom In increases the document size by 10 percentage points.
2. Zoom Out decreases the document size by 10 percentage points.
3. Reset restores the original 100% size.

## Inline and fenced code

Inline code such as `cargo test` should resize with the paragraph.

```rust
fn document_size(percent: u16) -> u16 {
    percent.clamp(50, 200)
}
```

## Table

| Action | Expected result |
| --- | --- |
| Zoom In | Larger document typography |
| Zoom Out | Smaller document typography |
| Reset | Original `100%` typography |

Switch to writable mode and verify that the editor text and line numbers resize together.
