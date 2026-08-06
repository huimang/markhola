use super::implementation::{
    AlertLabels, extract_title, highlight_assets, render_html,
    render_html_with_image_resolver_and_alert_labels, resolve_syntax,
};

#[test]
fn extracts_first_heading_as_title() {
    let markdown = "intro\n# Hello\n## Next";
    assert_eq!(extract_title(markdown).as_deref(), Some("Hello"));
}

#[test]
fn renders_tables() {
    let markdown = "| A | B |\n| - | - |\n| 1 | 2 |";
    let html = render_html(markdown);
    assert!(html.contains("<table>"));
    assert!(html.contains("<td>1</td>"));
    assert!(html.contains("class=\"markdown-table-region\""));
    assert!(html.contains("role=\"region\""));
    assert!(html.contains("aria-label=\"Scrollable table\""));
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("</table>\n</div>"));
}

#[test]
fn renders_highlighted_code_blocks_with_line_numbers() {
    let markdown = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
    let html = render_html(markdown);

    assert!(html.contains("class=\"code-block\""));
    assert!(html.contains("data-language=\"rust\""));
    assert!(html.contains("class=\"code-block__badge\">rust</div>"));
    assert!(html.contains("class=\"code-block__line-number\">1</span>"));
    assert!(html.contains("class=\"code-block__line-number\">3</span>"));
    assert!(html.contains("class=\"code-syntax code-syntax--keyword\""));
    assert!(html.contains("class=\"code-syntax code-syntax--string\""));
    assert!(!html.contains("style=\""));
    assert!(!html.contains("color:#"));
}

#[test]
fn branded_syntax_theme_covers_keywords_strings_constants_and_comments() {
    let markdown = "```rust\npub fn palette() -> &'static str {\n    // MarkHola\n    let count = 42;\n    \"green\"\n}\n```";
    let html = render_html(markdown);

    assert!(html.contains("code-syntax--keyword"));
    assert!(html.contains("code-syntax--string"));
    assert!(html.contains("code-syntax--constant"));
    assert!(html.contains("code-syntax--comment"));
    assert!(!html.contains("font-weight:bold"));
}

#[test]
fn renders_mermaid_blocks_separately_from_code_highlighting() {
    let markdown = "```mermaid\nflowchart TD\nA --> B\n```";
    let html = render_html(markdown);

    assert!(html.contains("class=\"mermaid-block\""));
    assert!(html.contains("class=\"mermaid-block__diagram\""));
    assert!(html.contains("flowchart TD"));
    assert!(!html.contains("class=\"code-block\""));
}

#[test]
fn renders_inline_and_display_math() {
    let markdown = "Inline math $e^{i\\pi}+1=0$.\n\n$$\\int_0^1 x^2 dx = \\frac{1}{3}$$";
    let html = render_html(markdown);

    assert!(html.contains("class=\"math math-inline\""));
    assert!(html.contains("e^{i\\pi}+1=0"));
    assert!(html.contains("class=\"math math-display\""));
    assert!(html.contains("\\int_0^1 x^2 dx = \\frac{1}{3}"));
}

#[test]
fn renders_fenced_math_blocks_separately_from_code_highlighting() {
    let markdown = "```math\n\\left( \\sum_{k=1}^n a_k b_k \\right)^2\n```";
    let html = render_html(markdown);

    assert!(html.contains("class=\"math-block\""));
    assert!(html.contains("class=\"math-block__formula\""));
    assert!(html.contains("\\left( \\sum_{k=1}^n a_k b_k \\right)^2"));
    assert!(!html.contains("class=\"code-block\""));
}

#[test]
fn falls_back_safely_for_unknown_languages() {
    let markdown = "```unknownlang\n<tag>\n```";
    let html = render_html(markdown);

    assert!(html.contains("data-language=\"unknownlang\""));
    assert!(html.contains("&lt;tag&gt;"));
    assert!(!html.contains("<tag>"));
}

#[test]
fn resolves_typescript_swift_and_kotlin_syntaxes() {
    let syntax_set = &highlight_assets().syntax_set;

    assert_eq!(
        resolve_syntax(syntax_set, "typescript").map(|syntax| syntax.name.as_str()),
        Some("JavaScript")
    );
    assert_eq!(
        resolve_syntax(syntax_set, "swift").map(|syntax| syntax.name.as_str()),
        Some("Rust")
    );
    assert_eq!(
        resolve_syntax(syntax_set, "kotlin").map(|syntax| syntax.name.as_str()),
        Some("Java")
    );
}

#[test]
fn resolves_alias_tokens_for_cpp_bash_and_yaml() {
    let syntax_set = &highlight_assets().syntax_set;

    assert_eq!(
        resolve_syntax(syntax_set, "cpp").map(|syntax| syntax.name.as_str()),
        Some("C++")
    );
    assert_eq!(
        resolve_syntax(syntax_set, "bash").map(|syntax| syntax.name.as_str()),
        Some("Bourne Again Shell (bash)")
    );
    assert_eq!(
        resolve_syntax(syntax_set, "yaml").map(|syntax| syntax.name.as_str()),
        Some("YAML")
    );
}

#[test]
fn preserves_blank_lines_in_code_blocks() {
    let markdown = "```text\nalpha\n\nomega\n```";
    let html = render_html(markdown);

    assert_eq!(
        html.matches("class=\"code-block__line-number\">").count(),
        3
    );
    assert_eq!(html.matches("class=\"code-block__line\">").count(), 3);
}

#[test]
fn leaves_inline_code_unchanged() {
    let html = render_html("Use `cargo test`.");

    assert!(html.contains("<code>cargo test</code>"));
    assert!(!html.contains("code-block__badge"));
}

#[test]
fn example_languages_keeps_mainstream_highlight_blocks() {
    let html = render_html(include_str!("../../examples/languages.md"));

    assert!(html.contains("data-language=\"typescript\""));
    assert!(html.contains("data-language=\"swift\""));
    assert!(html.contains("data-language=\"kotlin\""));
    assert!(html.contains("class=\"code-block__line-number\">1</span>"));
}

#[test]
fn example_mermaid_keeps_mermaid_render_containers() {
    let source = include_str!("../../examples/mermaid.md");
    let html = render_html(source);

    assert!(html.contains("class=\"mermaid-block\""));
    assert!(html.contains("class=\"mermaid-block__diagram\""));
    assert!(source.contains(r#"A[First line\nSecond line]"#));
    assert!(html.contains(r#"First line\nSecond line"#));
}

#[test]
fn example_math_keeps_math_render_containers() {
    let html = render_html(include_str!("../../examples/math.md"));

    assert!(html.contains("class=\"math math-inline\""));
    assert!(html.contains("class=\"math math-display\""));
    assert!(html.contains("class=\"math-block\""));
}

#[test]
fn renders_toc_when_placeholder_present() {
    let markdown = "# Title\n\n[toc]\n\n## Section A\n\n### Subsection";
    let html = render_html(markdown);

    assert!(html.contains("class=\"toc\""));
    assert!(html.contains("href=\"#title\""));
    assert!(html.contains("href=\"#section-a\""));
    assert!(html.contains("href=\"#subsection\""));
}

#[test]
fn does_not_render_toc_without_placeholder() {
    let markdown = "# Title\n\n## Section";
    let html = render_html(markdown);

    assert!(!html.contains("class=\"toc\""));
}

#[test]
fn renders_angle_bracket_shorthand_links() {
    let html = render_html("<xx>");

    assert!(html.contains("<a href=\"xx\">xx</a>"));
}

#[test]
fn renders_angle_bracket_paths_and_urls_as_links() {
    let html = render_html("<README.md>\n<docs/intro.md>\n<https://example.com>");

    assert!(html.contains("<a href=\"README.md\">README.md</a>"));
    assert!(html.contains("<a href=\"docs/intro.md\">docs/intro.md</a>"));
    assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
}

#[test]
fn keeps_angle_brackets_literal_inside_inline_code() {
    let html = render_html("`<README.md>`");

    assert!(html.contains("<code>&lt;README.md&gt;</code>"));
    assert!(!html.contains("href=\"README.md\""));
}

#[test]
fn keeps_angle_brackets_literal_inside_fenced_code() {
    let html = render_html("```text\n<README.md>\n```");

    assert!(html.contains("&lt;README.md&gt;"));
    assert!(!html.contains("href=\"README.md\""));
}

#[test]
fn does_not_turn_html_tags_into_links() {
    let html = render_html("<b>bold</b>\n<div>block</div>");

    assert!(html.contains("<b>bold</b>"));
    assert!(html.contains("<div>block</div>"));
    assert!(!html.contains("href=\"b\""));
    assert!(!html.contains("href=\"div\""));
}

#[test]
fn example_angle_bracket_links_keeps_rendered_and_literal_cases_separate() {
    let html = render_html(include_str!("../../examples/angle-bracket-links.md"));

    assert!(html.contains("<a href=\"README.md\">README.md</a>"));
    assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
    assert!(html.contains("<code>&lt;README.md&gt;</code>"));
    assert!(html.contains("&lt;README.md&gt;"));
    assert!(html.contains("<b>bold-like tag</b>"));
}

#[test]
fn export_verification_example_keeps_disposable_copy_and_source_preservation_instructions() {
    let source = include_str!("../../examples/v0.9.1-png-export-and-save-as.md");
    let html = render_html(source);

    assert!(source.contains("Do not edit or save this canonical file during verification."));
    assert!(source.contains("Copy `examples/v0.9.1-png-export-and-save-as.md` and `examples/assets/` into a disposable"));
    assert!(source.contains("The Git hashes of this canonical file and `examples/assets/diagram.svg`"));
    assert!(source.contains("Save As preserves the original copied source"));
    assert!(source.contains("Confirm the old source hash is unchanged."));
    assert!(source.contains("Confirm the new file contains the current full Markdown source."));
    assert!(source.contains("![MarkHola local export diagram](./assets/diagram.svg)"));
    assert!(source.contains("```mermaid"));
    assert!(source.contains("$E = mc^2$"));
    assert!(source.contains("| Surface | Light expectation | Dark expectation |"));
    assert!(source.contains("\n---\n"));
    assert!(html.contains("class=\"mermaid-block\""));
    assert!(html.contains("class=\"math math-inline\""));
    assert!(html.contains("<table>"));
    assert!(html.contains("assets/diagram.svg"));
}

#[test]
fn renders_referenced_footnotes_in_first_reference_order_with_distinct_backlinks() {
    let html =
        render_html("Second[^b], first[^a], and second again[^b].\n\n[^a]: Alpha.\n[^b]: Beta.");

    assert!(html.contains("id=\"markhola-footnote-ref-1-1\""));
    assert!(html.contains("id=\"markhola-footnote-ref-1-2\""));
    assert!(html.contains("id=\"markhola-footnote-ref-2-1\""));
    assert!(html.contains("href=\"#markhola-footnote-definition-1\" aria-label=\"Footnote 1\""));
    assert!(html.contains("href=\"#markhola-footnote-ref-1-1\""));
    assert!(html.contains("href=\"#markhola-footnote-ref-1-2\""));
    assert!(html.find("Beta.").unwrap() < html.find("Alpha.").unwrap());
}

#[test]
fn renders_only_referenced_definitions_and_preserves_safe_markdown_content() {
    let html = render_html(
        "Text[^used].\n\n[^unused]: Hidden.\n[^used]: **Strong** with $x^2$, a list:\n\n    - item\n\n    ![local](asset.svg)",
    );

    assert!(html.contains("<strong>Strong</strong>"));
    assert!(html.contains("class=\"math math-inline\""));
    assert!(html.contains("<li>item</li>"));
    assert!(html.contains("src=\"asset.svg\""));
    assert!(!html.contains("Hidden."));
}

#[test]
fn footnotes_fail_safe_for_missing_nested_and_duplicate_definitions() {
    let html = render_html(
        "Missing[^missing] and used[^one].\n\n[^one]: First with nested[^two].\n[^one]: Duplicate.\n[^two]: Nested target.",
    );

    assert!(html.contains("[^missing]"));
    assert!(html.contains("First with nested[^two]."));
    assert!(!html.contains("Duplicate."));
    assert!(!html.contains("Nested target."));
}

#[test]
fn footnote_anchor_namespace_is_separate_from_heading_ids() {
    let html = render_html(
        "# Markhola Footnote Definition 1\n\nText[^note].\n\n[^note]: Note.",
    );

    assert!(html.contains("id=\"heading-markhola-footnote-definition-1\""));
    assert!(html.contains("id=\"markhola-footnote-definition-1\""));
    assert!(html.contains("aria-label=\"Footnotes\""));
    assert!(html.contains("aria-label=\"Back to footnote 1 reference 1\""));
}

#[test]
fn footnotes_escape_raw_html_and_do_not_activate_custom_nested_content() {
    let html = render_html(
        "Safe[^note].\n\n[^note]: <script>alert('no')</script> and <span>plain</span>.",
    );

    assert!(!html.contains("<script>"));
    assert!(!html.contains("<span>plain</span>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&lt;span&gt;plain&lt;/span&gt;"));
}

#[test]
fn renders_every_alert_type_with_accessible_region_and_localizable_label() {
    for (marker, token, label) in [
        ("NOTE", "note", "Note"),
        ("TIP", "tip", "Tip"),
        ("IMPORTANT", "important", "Important"),
        ("WARNING", "warning", "Warning"),
        ("CAUTION", "caution", "Caution"),
    ] {
        let html = render_html(&format!("> [!{marker}]\n> Body."));

        assert!(html.contains(&format!("data-alert=\"{token}\"")), "{marker}");
        assert!(html.contains("class=\"mh-alert\""), "{marker}");
        assert!(html.contains("role=\"note\""), "{marker}");
        assert!(
            html.contains("aria-labelledby=\"markhola-alert-1\""),
            "{marker}"
        );
        assert!(
            html.contains(&format!("class=\"mh-alert-label\">{label}</span>")),
            "{marker}"
        );
        assert!(html.contains("aria-hidden=\"true\""), "{marker}");
        assert!(html.contains("<p>Body.</p>"), "{marker}");
    }
}

#[test]
fn alert_markers_accept_case_and_trailing_whitespace() {
    for source in [
        "> [!tip]\n> Body.",
        "> [!TiP]\n> Body.",
        "> [!TIP]  \n> Body.",
        "> [!TIP]\t\n> Body.",
        ">[!TIP]\n> Body.",
        "   > [!TIP]\n   > Body.",
        "> [!TIP]\n>\n> Body.",
    ] {
        assert!(
            render_html(source).contains("data-alert=\"tip\""),
            "{source:?}"
        );
    }
}

#[test]
fn ordinary_blockquotes_stay_unchanged_for_every_rejected_marker() {
    let plain = render_html("> Body.");
    assert_eq!(plain, "<blockquote>\n<p>Body.</p>\n</blockquote>\n");

    for source in [
        ">  [!NOTE]\n> Body.",
        "> [!TIP] inline\n> Body.",
        "> [!HINT]\n> Body.",
        "> First.\n> [!WARNING]\n> Body.",
        "> `[!NOTE]`\n> Body.",
        "> ［!NOTE］\n> Body.",
    ] {
        let html = render_html(source);
        assert!(html.starts_with("<blockquote>"), "{source:?}");
        assert!(!html.contains("mh-alert"), "{source:?}");
    }

    assert!(!render_html("```\n> [!CAUTION]\n> Body.\n```").contains("mh-alert"));
    assert!(!render_html("    > [!CAUTION]\n    > Body.").contains("mh-alert"));
}

#[test]
fn alert_without_content_degrades_to_an_ordinary_blockquote() {
    assert_eq!(render_html("> [!NOTE]"), render_html(">"));
    assert!(!render_html("> [!NOTE]").contains("mh-alert"));
}

#[test]
fn nested_alert_markers_degrade_to_ordinary_blockquotes() {
    let html = render_html("> [!NOTE]\n> Outer.\n>\n> > [!TIP]\n> > Inner.");

    assert_eq!(html.matches("class=\"mh-alert\"").count(), 1);
    assert!(html.contains("data-alert=\"note\""));
    assert!(!html.contains("data-alert=\"tip\""));
    assert!(html.contains("<blockquote>\n<p>Inner.</p>\n</blockquote>"));
}

#[test]
fn alert_identifiers_increment_and_stay_separate_from_heading_ids() {
    let html = render_html("> [!NOTE]\n> One.\n\n> [!TIP]\n> Two.");
    assert!(html.contains("id=\"markhola-alert-1\""));
    assert!(html.contains("id=\"markhola-alert-2\""));

    let collision = render_html("# Markhola Alert 1\n\n> [!NOTE]\n> Body.");
    assert!(collision.contains("id=\"heading-markhola-alert-1\""));
    assert_eq!(collision.matches("id=\"markhola-alert-1\"").count(), 1);
}

#[test]
fn alert_content_reuses_existing_safe_rendering() {
    let html = render_html(
        "> [!IMPORTANT]\n> **Strong** and $x^2$ with [link](README.md).\n>\n> - item\n>\n> ![local](asset.svg)\n>\n> ```rust\n> let x = 1;\n> ```",
    );

    assert!(html.contains("<strong>Strong</strong>"));
    assert!(html.contains("class=\"math math-inline\""));
    assert!(html.contains("href=\"README.md\""));
    assert!(html.contains("<li>item</li>"));
    assert!(html.contains("src=\"asset.svg\""));
    assert!(html.contains("class=\"code-block\""));

}

#[test]
fn alert_content_does_not_change_the_document_raw_html_policy() {
    // Footnotes deliberately narrow their content model and downgrade raw HTML to text. Alerts are
    // an ordinary blockquote surface, so they must neither widen nor narrow the document policy.
    for fragment in ["<b>bold</b>", "<div>block</div>", "<script>alert('no')</script>"] {
        let inside = render_html(&format!("> [!NOTE]\n> {fragment}"));
        let outside = render_html(fragment);
        let escaped = fragment.replace('<', "&lt;").replace('>', "&gt;");

        assert_eq!(
            inside.contains(fragment),
            outside.contains(fragment),
            "{fragment:?}"
        );
        assert_eq!(
            inside.contains(&escaped),
            outside.contains(&escaped),
            "{fragment:?}"
        );
    }
}

#[test]
fn export_paths_inject_localized_alert_labels_without_scripting() {
    let labels = AlertLabels {
        note: "备注",
        tip: "提示",
        important: "重要",
        warning: "警告",
        caution: "注意",
    };
    let html = render_html_with_image_resolver_and_alert_labels(
        "> [!WARNING]\n> Body.\n\n> [!TIP]\n> Body.",
        |_| None,
        labels,
    );

    assert!(html.contains("class=\"mh-alert-label\">警告</span>"));
    assert!(html.contains("class=\"mh-alert-label\">提示</span>"));
    assert!(html.contains("data-alert=\"warning\""));
    assert!(!html.contains(">Warning<"));

    // The default entry point stays English so offline CLI and socket output remain deterministic.
    assert!(render_html("> [!WARNING]\n> Body.").contains("class=\"mh-alert-label\">Warning</span>"));
}
