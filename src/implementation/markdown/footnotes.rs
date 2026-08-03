use std::collections::HashMap;

use pulldown_cmark::{html, CowStr, Event, Tag, TagEnd};

use super::{escape_html, escape_html_attribute};

#[derive(Debug)]
struct Definition<'a> {
    events: Vec<Event<'a>>,
}

#[derive(Debug)]
struct Reference {
    number: usize,
    count: usize,
}

pub(super) fn rewrite<'a>(events: impl IntoIterator<Item = Event<'a>>) -> (Vec<Event<'a>>, String) {
    let mut body = Vec::new();
    let mut definitions = HashMap::<String, Definition<'a>>::new();
    let mut active_definition: Option<(String, Vec<Event<'a>>)> = None;

    for event in events {
        match event {
            Event::Start(Tag::FootnoteDefinition(label)) if active_definition.is_none() => {
                active_definition = Some((label.into_string(), Vec::new()));
            }
            Event::End(TagEnd::FootnoteDefinition) if active_definition.is_some() => {
                let (label, definition_events) = active_definition.take().expect("checked above");
                if !definitions.contains_key(&label) {
                    definitions.insert(
                        label,
                        Definition {
                            events: definition_events,
                        },
                    );
                }
            }
            Event::FootnoteReference(label) if active_definition.is_some() => {
                // Nested footnotes are deliberately outside the supported syntax. Preserve them
                // as readable text instead of creating a second semantic footnote graph.
                active_definition
                    .as_mut()
                    .expect("checked above")
                    .1
                    .push(Event::Text(CowStr::Boxed(
                        format!("[^{}]", label).into_boxed_str(),
                    )));
            }
            Event::Html(source) | Event::InlineHtml(source) if active_definition.is_some() => {
                // Raw HTML is not part of the accepted footnote content model. Keeping it as text
                // prevents a definition from introducing script or custom embedded behavior.
                active_definition
                    .as_mut()
                    .expect("checked above")
                    .1
                    .push(Event::Text(source));
            }
            event if active_definition.is_some() => active_definition
                .as_mut()
                .expect("checked above")
                .1
                .push(event),
            event => body.push(event),
        }
    }

    // An unterminated definition is a parser-level fallback. Keep its content readable and do not
    // manufacture an incomplete footnote section.
    if let Some((label, events)) = active_definition {
        body.push(Event::Text(CowStr::Boxed(
            format!("[^{}]: ", label).into_boxed_str(),
        )));
        body.extend(events);
    }

    let mut references = HashMap::<String, Reference>::new();
    let mut referenced_order = Vec::new();
    for event in &body {
        if let Event::FootnoteReference(label) = event {
            let label = label.as_ref();
            if !definitions.contains_key(label) {
                continue;
            }
            let next_number = referenced_order.len() + 1;
            references.entry(label.to_owned()).or_insert_with(|| {
                referenced_order.push(label.to_owned());
                Reference {
                    number: next_number,
                    count: 0,
                }
            });
        }
    }

    let body = body
        .into_iter()
        .map(|event| match event {
            Event::FootnoteReference(label) => {
                let label_text = label.as_ref();
                let Some(reference) = references.get_mut(label_text) else {
                    return Event::Html(
                        format!(
                            "<span class=\"footnote-reference footnote-reference--missing\" aria-label=\"Missing footnote: {}\">[^{}]</span>",
                            escape_html_attribute(label_text),
                            escape_html(label_text)
                        )
                        .into(),
                    );
                };
                reference.count += 1;
                Event::Html(
                    format!(
                        "<sup class=\"footnote-reference\" id=\"markhola-footnote-ref-{}-{}\"><a href=\"#markhola-footnote-definition-{}\" aria-label=\"Footnote {}\">[{}]</a></sup>",
                        reference.number,
                        reference.count,
                        reference.number,
                        reference.number,
                        reference.number
                    )
                    .into(),
                )
            }
            event => event,
        })
        .collect();

    let section = render_section(&referenced_order, &definitions, &references);
    (body, section)
}

fn render_section(
    referenced_order: &[String],
    definitions: &HashMap<String, Definition<'_>>,
    references: &HashMap<String, Reference>,
) -> String {
    if referenced_order.is_empty() {
        return String::new();
    }

    let mut output = String::from(
        "<section class=\"footnotes\" aria-label=\"Footnotes\"><hr><ol class=\"footnotes__list\">\n",
    );
    for label in referenced_order {
        let Some(definition) = definitions.get(label) else {
            continue;
        };
        let Some(reference) = references.get(label) else {
            continue;
        };
        output.push_str(&format!(
            "<li class=\"footnotes__item\" id=\"markhola-footnote-definition-{}\">",
            reference.number
        ));
        html::push_html(&mut output, definition.events.clone().into_iter());
        output.push_str("<span class=\"footnotes__backlinks\" aria-label=\"Back to reference\">");
        for occurrence in 1..=reference.count {
            output.push_str(&format!(
                "<a class=\"footnotes__backlink\" href=\"#markhola-footnote-ref-{}-{}\" aria-label=\"Back to footnote {} reference {}\">Back {}</a>",
                reference.number,
                occurrence,
                reference.number,
                occurrence,
                occurrence
            ));
        }
        output.push_str("</span></li>\n");
    }
    output.push_str("</ol></section>\n");
    output
}
