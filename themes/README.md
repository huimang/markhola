# Themes

- `default/layout.css`: current default app shell layout theme
- `dark/layout.css`: dark reading shell theme
- `light/layout.css`: bright neutral shell theme

MarkHola loads the selected theme from `themes/<theme-name>/layout.css` at runtime when available.
In development, edit the repository `themes/<theme-name>/layout.css` directly.
In the packaged macOS app, the same theme directories are copied into `MarkHola.app/Contents/Resources/themes/`.

Theme and syntax-color changes must follow
[`docs/visual-design-guidelines.md`](../docs/visual-design-guidelines.md), including
contrast calculations and real-rendering checks for the affected Default and Dark states.
