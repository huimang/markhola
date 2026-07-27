# MarkHola Brand Colors

Use this file to verify the MarkHola palette and table header styling in `v0.8.1`.

| Color | Hex | Intended role |
| --- | --- | --- |
| MarkHola Violet | `#6657E8` | Primary brand violet |
| Violet Mid | `#ACA4F4` | Violet transition |
| Violet Tint | `#F2F0FF` | Light violet surface |
| MarkHola Green | `#17B890` | Primary brand green |
| Green Mid | `#81D9C3` | Green transition |
| Green Tint | `#EAF9F5` | Markdown table header |

The table header should use the same Green Tint background in the Default and Dark app themes.

## Link and quote

This [MarkHola link](https://github.com/huimang/markhola) should use MarkHola Violet and change to MarkHola Green on hover.

> The quote border should use the green brand scale.

## Outline levels

Use these headings to verify the Outline panel colors.

### Violet interactions

Tab, navigation, size controls, and Outline item hover states use the violet scale.

#### Green active states

The active Tab underline, mode indicator, Outline toggle, and table header use the green scale.

## Branded syntax highlighting

```rust
pub struct Palette {
    name: &'static str,
    violet: u32,
    green: u32,
}

pub fn markhola_palette() -> Palette {
    // Keywords, types, strings, constants, and comments should remain distinct.
    Palette {
        name: "MarkHola",
        violet: 0x6657E8,
        green: 0x17B890,
    }
}
```

Verify the purple-derived code surface, the separate line-number gutter, the green language badge, and the branded syntax colors.
