use std::sync::Arc;

use egui::{Galley, Ui};

/// Lay out normal-whitespace description text without emergency word splitting.
pub(super) fn description(ui: &Ui, natural: Arc<Galley>, available: f32) -> (Arc<Galley>, f32) {
    let text = &natural.job.text;
    if text.is_empty() || natural.size().x <= available {
        return (natural, available);
    }
    let unicode: Vec<_> = unicode_linebreak::linebreaks(text)
        .map(|(end, _)| end)
        .collect();
    let mut ends = Vec::new();
    let mut previous: Option<char> = None;
    let mut before_previous = None;
    for (offset, current) in text.char_indices() {
        if let Some(before) = previous {
            let allowed = if before.is_ascii_graphic() && current.is_ascii_graphic() {
                ascii_break(before_previous, before, current)
            } else {
                unicode.binary_search(&offset).is_ok()
            };
            if allowed {
                ends.push(offset);
            }
        }
        before_previous = previous;
        previous = Some(current);
    }
    ends.push(text.len());
    let shape = |start: usize, end: usize| {
        // CSS hangs collapsed line-end spaces; they must not consume width or
        // make the following word lose an otherwise valid break opportunity.
        let mut job = (*natural.job).clone();
        job.text = text[start..end].trim_end_matches(' ').to_owned();
        job.sections[0].byte_range =
            egui::text::ByteIndex(0)..egui::text::ByteIndex(job.text.len());
        ui.painter().layout_job(job)
    };
    let mut start = 0;
    let mut minimum = 0.0_f32;
    for &end in &ends {
        minimum = minimum.max(shape(start, end).size().x);
        start = end;
    }
    let width = available.max(minimum);
    if natural.size().x <= width {
        return (natural, width);
    }
    let mut lines = Vec::new();
    let mut fitting = None;
    let mut start = 0;
    let mut index = 0;
    while index < ends.len() {
        let line = shape(start, ends[index]);
        if line.size().x > width
            && let Some(last) = fitting.take()
        {
            lines.push(last);
            start = ends[index - 1];
            continue;
        }
        fitting = Some(line);
        index += 1;
    }
    lines.push(fitting.expect("nonempty description has an end boundary"));
    let mut galley = Galley::concat(Arc::clone(&natural.job), &lines, ui.pixels_per_point());
    // concat normally rejoins hard paragraphs. These are soft CSS wraps, not
    // inserted source newlines; accessible text remains the complete input.
    for row in &mut galley.rows {
        row.ends_with_newline = false;
    }
    (Arc::new(galley), width)
}

// Printable ASCII compatibility follows Chromium's BSD-licensed
// LineBreakData::FillAscii. Numeric hyphen context is separately checked against
// Chrome's measured rendering; this is not a slash-specific URL heuristic.
// Copyright 2016 The Chromium Authors. BSD-3-Clause; see THIRD_PARTY_NOTICES.md.
fn ascii_break(before_previous: Option<char>, before: char, current: char) -> bool {
    if before == '-' && current.is_ascii_digit() {
        return before_previous.is_some_and(|ch| ch.is_ascii_alphanumeric());
    }
    if before.is_ascii_alphanumeric()
        || matches!(
            before,
            '$' | '\'' | '(' | '/' | '<' | '@' | '[' | '^' | '_' | '`' | '{'
        )
        || matches!(
            current,
            '!' | ')' | ',' | '.' | '/' | ':' | ';' | '?' | ']' | '}'
        )
        || (before == '?' && matches!(current, '"' | '\''))
        || (before == '-' && current == '$')
    {
        return false;
    }
    matches!(before, '-' | '?') || matches!(current, '(' | '<' | '[' | '{')
}
