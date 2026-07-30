# Themes

- `default/layout.css`: current default app shell layout theme
- `dark/layout.css`: dark reading shell theme

MarkHola loads the selected theme from `themes/<theme-name>/layout.css` at runtime when available.
`Follow System` resolves the macOS Light appearance to the user-visible Light theme and Dark
appearance to Dark. Light retains the canonical internal key and directory name `default`.
In development, edit the repository `themes/<theme-name>/layout.css` directly.
In the packaged macOS app, the same theme directories are copied into `MarkHola.app/Contents/Resources/themes/`.

Theme and syntax-color changes must follow
[`docs/visual-design-guidelines.md`](../docs/visual-design-guidelines.md), including
contrast calculations and real-rendering checks for the affected Light and Dark states.
