# Theme Showcase

Use this file to verify the `View > Theme` menu in `v0.8.1`.

## Paragraph

This paragraph includes **bold text**, *italic text*, and a [link](https://example.com).

The link should use the MarkHola theme colors without an underline, including when hovered or focused.

## Checklist

- [x] Theme switch keeps document content visible
- [ ] Verify this file in writable mode too

## Code

```rust
fn palette(name: &str) -> &'static str {
    match name {
        "dark" => "night",
        "system" => "follow macOS",
        _ => "default",
    }
}
```

## Table

| Theme | Intent |
| --- | --- |
| default | bright low-stimulation reading shell |
| dark | low-light reading |
| system | follows the current macOS Light or Dark appearance |

The code block should use a pale gray-violet surface in Default and a soft dark gray-violet
surface in Dark, with restrained syntax colors and readable line numbers.
