# MarkHola Visual Design Guidelines

## Purpose and Authority

This document is the canonical repository reference for MarkHola color, theme, long-reading, and
visual-comfort decisions. Theme CSS, native macOS surfaces, Markdown rendering, syntax highlighting,
examples, and future visual work must follow it.

Version-scoped implementation designs and test plans belong under the Git-ignored `drafts/`
directory. Durable visual rules must be promoted here instead of remaining scattered across version
drafts.

MarkHola supports sustained Markdown reading and editing. Its visual design must preserve brand
identity and accessibility while reducing avoidable luminance jumps, excessive saturation, and
localized glare. Visual comfort does not claim to prevent or treat eye conditions; it means reducing
avoidable visual load while preserving readability for people with low vision.

## Brand Color System

The Violet, Green, and Gray scales are the shared source of truth across Default and Dark. Every
defined color includes an HTML swatch so the specification shows the actual color instead of relying
on its name or HEX value alone.

| Scale | Token | Value | HTML preview | Primary role |
| --- | --- | --- | --- | --- |
| Violet | `--markhola-violet` | `#6657E8` | <span title="MarkHola Violet #6657E8" aria-label="MarkHola Violet #6657E8" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#6657E8;border:1px solid #94A3B8;"></span> | Primary interaction, links, focus |
| Violet | `--markhola-violet-mid` | `#ACA4F4` | <span title="Violet Mid #ACA4F4" aria-label="Violet Mid #ACA4F4" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#ACA4F4;border:1px solid #94A3B8;"></span> | Intermediate borders and syntax |
| Violet | `--markhola-violet-tint` | `#F2F0FF` | <span title="Violet Tint #F2F0FF" aria-label="Violet Tint #F2F0FF" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#F2F0FF;border:1px solid #94A3B8;"></span> | Soft interaction background |
| Violet | `--markhola-violet-subtle` | `#F9F8FF` | <span title="Violet Subtle #F9F8FF" aria-label="Violet Subtle #F9F8FF" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#F9F8FF;border:1px solid #94A3B8;"></span> | Large subtle Violet surface |
| Green | `--markhola-green` | `#17B890` | <span title="MarkHola Green #17B890" aria-label="MarkHola Green #17B890" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#17B890;border:1px solid #94A3B8;"></span> | Active, selected, confirmed state |
| Green | `--markhola-green-mid` | `#81D9C3` | <span title="Green Mid #81D9C3" aria-label="Green Mid #81D9C3" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#81D9C3;border:1px solid #94A3B8;"></span> | Active borders and syntax |
| Green | `--markhola-green-tint` | `#EAF9F5` | <span title="Green Tint #EAF9F5" aria-label="Green Tint #EAF9F5" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#EAF9F5;border:1px solid #94A3B8;"></span> | Tabs, table headers, active background |
| Green | `--markhola-green-subtle` | `#FAFEFD` | <span title="Green Subtle #FAFEFD" aria-label="Green Subtle #FAFEFD" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#FAFEFD;border:1px solid #94A3B8;"></span> | Default document surface |
| Gray | `--markhola-gray-strong` | `#1E293B` | <span title="Gray Strong #1E293B" aria-label="Gray Strong #1E293B" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#1E293B;border:1px solid #94A3B8;"></span> | Strong neutral text or dark surface |
| Gray | `--markhola-gray` | `#475569` | <span title="Gray #475569" aria-label="Gray #475569" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#475569;border:1px solid #94A3B8;"></span> | Neutral application text |
| Gray | `--markhola-gray-mid` | `#94A3B8` | <span title="Gray Mid #94A3B8" aria-label="Gray Mid #94A3B8" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#94A3B8;border:1px solid #64748B;"></span> | Secondary neutral content |
| Gray | `--markhola-gray-tint` | `#E2E8F0` | <span title="Gray Tint #E2E8F0" aria-label="Gray Tint #E2E8F0" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#E2E8F0;border:1px solid #94A3B8;"></span> | Title-bar material and neutral boundary |
| Gray | `--markhola-gray-subtle` | `#F8FAFC` | <span title="Gray Subtle #F8FAFC" aria-label="Gray Subtle #F8FAFC" style="display:inline-block;width:72px;height:20px;vertical-align:middle;background:#F8FAFC;border:1px solid #94A3B8;"></span> | Large neutral surface |

Mid colors are stable transitions between the primary color and its Tint. Subtle colors are for
large surfaces where even a Tint would be too visually dominant.

### Semantic Use

- Violet represents general interaction, links, keyboard focus, and hover emphasis.
- Green represents the current, active, selected, or confirmed state.
- Gray represents neutral application chrome, metadata, title bars, and footers.
- Warning, danger, deletion, and search-result colors retain their own semantic tokens. Do not
  replace them with brand colors.
- Backgrounds, body text, and separators use theme semantic tokens such as `--bg`, `--text`, and
  `--border`; brand colors do not replace every neutral color.
- Components must consume shared tokens rather than introduce a second local brand palette.

## Theme Model

The target theme model contains two rendering themes:

- Default, for the light appearance.
- Dark, for the dark appearance.

Follow System is a preference rather than a third CSS theme:

- macOS Light maps to Default.
- macOS Dark maps to Dark.
- Manual Default or Dark remains fixed until Follow System is selected again.

The separate Light theme is retired because its role substantially overlaps Default. Default and
Dark must each define an appropriate palette instead of sharing colors that only work in one
appearance.

## Surface Hierarchy

### Default

- Use a subtle Gray title-bar material so native chrome remains quiet.
- Use Green Tint for the selected native tab and Green Subtle for the document surface.
- Use the Gray scale for the footer and metadata.
- Keep the title bar, tab strip, document, and footer visually distinct without strong saturation or
  large luminance jumps.

### Dark

- Use low-saturation dark neutral or gray-violet surfaces rather than pure black.
- Preserve the same semantic hierarchy as Default while selecting colors that remain comfortable
  and legible in low-light conditions.
- Avoid luminous near-white text on large dark areas unless an increased-contrast mode requires it.

## Components and Interaction

### Native Tabs

- Keep native AppKit tab geometry and behavior.
- Use Green for the selected/current state and restrained Gray for surrounding chrome.
- Numbered shortcut accessories may show `⌘1` through `⌘9`; repeated `Tab 1` through `Tab 9` menu
  entries are not part of the visual model.

### Links

- Use Violet for normal Markdown links and Green for hover emphasis.
- Do not show an underline in default, visited, hover, focus, or active states.
- Preserve an accessible keyboard-focus indicator without reintroducing an underline.
- Do not use a third visited-link color outside the brand system.

### Outline

- Use Green Tint, Green Mid, and Green for the expanded/current Outline state.
- Use Violet Tint and Violet for item hover and keyboard focus.
- Keep headings, empty states, and inactive items neutral.

### Tables

- Use Green Tint for table headers with text that meets the relevant contrast requirement.
- Use straight table corners.
- Keep borders, padding, and alignment restrained so the header color does not dominate the page.

### Footer

- Use the Gray scale rather than Green.
- Keep path metadata on the left and document counts on the right.
- Use one consistent lower-brightness foreground color on both sides.
- Do not show debug status, loaded-state messages, Readonly, or Writable.

### Reading Surface

- Keep the document centered with a maximum reading width of 960 px.
- Use quiet large-area backgrounds and avoid decorative surfaces that compete with document content.
- Maintain clear hierarchy through spacing, typography, and restrained color rather than saturation
  alone.

## Visual Comfort and Accessibility

Use WCAG 2.2 AA and Apple Accessibility small-text guidance as the readability floor.

- Regular 14 px text must have at least `4.5:1` contrast.
- Required non-text boundaries, focus indicators, and state indicators must have at least `3:1`.
- Do not treat higher contrast as automatically better. After meeting the readability floor,
  control large-area luminance differences and color saturation.
- Validate with both calculated contrast and real rendering. Calculations prove the threshold;
  screenshots and manual inspection verify hierarchy, continuous-reading comfort, and transitions
  between adjacent surfaces.

References:

- [WCAG 2.2 Contrast Minimum](https://www.w3.org/TR/WCAG22/#contrast-minimum)
- [Apple Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility/)
- [Apple Sufficient Contrast Evaluation Criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/sufficient-contrast-evaluation-criteria/)

## Code Presentation

### Contrast Targets

- Keep regular 14 px code text around `6:1–8:1`; do not use `13:1–14:1` near-white text by default.
- Keep line numbers and other secondary text around `4.5:1–5.5:1`.
- Keep required non-text separators at least `3:1`.
- Keep language badges and other small text at least `4.5:1`.
- If a syntax color misses the minimum, adjust luminance or the background instead of relying on
  higher saturation.

### Default Code Palette

- Use a pale gray-violet or pale neutral code background.
- Avoid placing a large deep purple-black block abruptly inside the light document surface.
- Separate the code area, gutter, and document through gentle hierarchy rather than a strong
  light-versus-dark cut.

### Dark Code Palette

- Use a soft, low-saturation dark gray-violet background rather than pure black.
- Distinguish code from the Dark document surface without making the block appear luminous or
  isolated.

### Syntax Colors

- Use low-saturation gray-violet for regular code text.
- Use restrained derivatives of MarkHola Violet and Green for keywords, strings, comments, and
  constants.
- Reduce bold keyword emphasis so color, luminance, and weight do not all create strong emphasis at
  the same time.
- Avoid using several near-maximum-luminance colors in one code block.
- Design and validate separate Default and Dark code palettes instead of reusing one fixed dark
  palette.

## Increased Contrast

Default themes must meet minimum accessibility requirements while prioritizing visual comfort.
A future version may respond to macOS Increase Contrast with a separate variant:

- The increased-contrast variant may raise text, boundary, and state-indicator contrast.
- It must not redefine the visual-comfort targets of the standard palette.
- Standard and increased-contrast variants must each receive contrast calculations and real-device
  validation.

## Validation Requirements

Any color or visual-experience change must:

1. Identify the semantic token and avoid introducing an unnecessary one-off color.
2. List affected foreground/background pairs and calculate their actual contrast.
3. Check the affected states in both Default and Dark.
4. Use real screenshots for layout, luminance transitions, spacing, and visual hierarchy.
5. Check hover, focus, active, selected, visited, and disabled states as applicable.
6. Check HTML, PDF, print, or native surfaces when they consume the changed visual rule.
7. Keep local validation limited to the affected component and immediately coupled surfaces; reserve
   full regression for release validation.
