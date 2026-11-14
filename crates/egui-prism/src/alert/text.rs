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
    let format = &natural.job.sections[0].format;
    let shape = |start: usize, end: usize| {
        // Copy just this candidate, not the complete source job on every probe.
        // Hanging spaces are restored as zero-advance glyphs after fitting.
        let candidate = text[start..end].trim_end_matches(' ');
        let mut job = egui::text::LayoutJob::single_section(candidate.to_owned(), format.clone());
        job.break_on_newline = false;
        job.round_output_to_gui = natural.job.round_output_to_gui;
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
    // Keep the natural job and its private intrinsic metrics. Soft wrapping
    // changes rows/meshes, not the unwrapped dimensions or source character map.
    let mut galley = (*natural).clone();
    galley.rows.clear();
    galley.mesh_bounds = egui::Rect::NOTHING;
    galley.num_vertices = 0;
    galley.num_indices = 0;
    let space = natural
        .rows
        .iter()
        .flat_map(|row| &row.glyphs)
        .find(|glyph| glyph.chr == ' ')
        .copied();
    let line_height = format
        .line_height
        .expect("Alert sets an explicit line height");
    let mut widest = 0.0_f32;
    let mut first = 0;
    while first < ends.len() {
        let start = if first == 0 { 0 } else { ends[first - 1] };
        let mut last = first;
        let mut line = shape(start, ends[last]);
        // Bracket the first non-fitting boundary by doubling, then bisect.
        // This avoids shaping every growing prefix when min-content is large,
        // without starting each short line by copying the whole remaining text.
        let mut step = 1;
        let mut upper = ends.len();
        loop {
            let next = first.saturating_add(step).min(ends.len() - 1);
            if next <= last {
                break;
            }
            let candidate = shape(start, ends[next]);
            if candidate.size().x > width {
                upper = next;
                break;
            }
            last = next;
            line = candidate;
            if last + 1 == ends.len() {
                break;
            }
            step *= 2;
        }
        while last + 1 < upper {
            let middle = last + (upper - last) / 2;
            let candidate = shape(start, ends[middle]);
            if candidate.size().x <= width {
                last = middle;
                line = candidate;
            } else {
                upper = middle;
            }
        }
        let end = ends[last];
        let hanging = text[start..end].len() - text[start..end].trim_end_matches(' ').len();
        let mut placed = line.rows[0].clone();
        let row = Arc::make_mut(&mut placed.row);
        for _ in 0..hanging {
            let mut glyph = space.expect("a hanging ASCII space exists in the natural galley");
            glyph.pos.x = line.size().x;
            glyph.advance_width = 0.0;
            row.glyphs.push(glyph);
        }
        row.size.y = line_height;
        let y = galley.rows.len() as f32 * line_height;
        placed.pos.y = (y * ui.pixels_per_point()).round() / ui.pixels_per_point();
        placed.ends_with_newline = false;
        galley.mesh_bounds |= placed.visuals.mesh_bounds.translate(placed.pos.to_vec2());
        galley.num_vertices += placed.visuals.mesh.vertices.len();
        galley.num_indices += placed.visuals.mesh.indices.len();
        widest = widest.max(line.size().x);
        galley.rows.push(placed);
        first = last + 1;
    }
    galley.rect = egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(widest, galley.rows.len() as f32 * line_height),
    );
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
