# MarkHola Technical Architecture

## Current stack

MarkHola is primarily implemented in Rust 2024. The native macOS application shell,
window lifecycle, menus, tabs, and document integration use Tao together with the
`objc2` AppKit/Foundation bindings. Web content is rendered in an embedded WRY
WebView; on macOS this uses Apple's WKWebView/WebKit rather than Chromium. The
WebView hosts the Markdown reading surface and related H5-style HTML interactions.

The Markdown pipeline uses `pulldown-cmark` for Markdown-to-HTML conversion and
`syntect` for code highlighting. Mermaid and MathJax are bundled local rendering
assets used by the WebView and by the existing export preparation paths. PDF export
uses the macOS PDFKit/AppKit integration, while HTML export reuses the generated
HTML and bundled runtime assets.

The application also contains native document, save, tab, menu, theme, and local
asset-access layers. These layers communicate through the Tao event loop and
application-owned interfaces; new automation or protocol features must reuse the
same production document and export services instead of creating parallel behavior.

## Architecture maintenance

This document is the canonical tracked reference for the current technical stack
and core runtime boundaries. The Architect owns terminology and architecture
updates, and must keep this document aligned with accepted implementation changes.
Product scope and version commitments remain in `PLAN.MD`; product planning must
not be duplicated here.
