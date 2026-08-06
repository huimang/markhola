# Markdown Alerts

MarkHola renders GitHub-compatible Alerts. A marker is recognized only when it is the first
non-empty content in a blockquote.

## Supported types

> [!NOTE]
> Useful information that users should know, even when skimming content.

> [!TIP]
> Helpful advice for doing things better or more easily.

> [!IMPORTANT]
> Key information users need to know to achieve their goal.

> [!WARNING]
> Urgent info that needs immediate user attention to avoid problems.

> [!CAUTION]
> Advises about risks or negative outcomes of certain actions.

## Accepted marker forms

> [!tip]
> Markers are ASCII case-insensitive.

> [!TIP]  
> Trailing whitespace after the marker is accepted.

>[!TIP]
> A space after `>` is optional.

> [!TIP]
>
> Content may start after a blank quote line.

## Content inside an Alert

> [!IMPORTANT]
> Alerts reuse the existing safe rendering set: **strong**, `inline code`, a
> [link](README.md), and inline math $E = mc^2$.
>
> - list item one
> - list item two
>
> ```rust
> fn main() {
>     println!("code blocks work too");
> }
> ```
>
> Multiple paragraphs keep their spacing across page breaks in PDF and Print output.

## Rejected markers fall back to ordinary blockquotes

Each of the following stays an ordinary blockquote.

>  [!NOTE]
> Whitespace before the marker is not accepted.

> [!TIP] inline text
> A marker followed by content on the same line is not accepted.

> [!HINT]
> Unknown types are not accepted.

> First line.
> [!WARNING]
> A marker that is not the first non-empty content is not accepted.

> `[!NOTE]`
> A marker inside inline code is not accepted.

> ［!NOTE］
> Full-width brackets are not accepted.

> [!NOTE]

The line above has a marker but no content, so it stays an ordinary blockquote.

## Markers inside code are never Alerts

```
> [!CAUTION]
> This is code, not an Alert.
```

    > [!CAUTION]
    > An indented code block is not an Alert either.

## Nested Alerts are out of scope

> [!NOTE]
> The outer marker renders as an Alert.
>
> > [!TIP]
> > The inner marker degrades to an ordinary blockquote.
