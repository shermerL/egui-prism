use std::sync::Arc;

use egui::{Galley, Ui};

/// Fit visible words, leaving collapsed spaces hanging at soft breaks.
/// Short paragraphs minimize squared remaining width across the original line
/// count; shrinking a greedy wrap width cannot choose all balanced partitions.
pub(super) fn wrap(ui: &Ui, natural: Arc<Galley>, available: f32) -> Arc<Galley> {
    let text = &natural.job.text;
    if text.is_empty() {
        return natural;
    }
    let mut ends: Vec<_> = text
        .match_indices(' ')
        .map(|(offset, _)| offset + 1)
        .collect();
    ends.push(text.len());
    let format = &natural.job.sections[0].format;
    let shape = |first: usize, end: usize| {
        let start = if first == 0 { 0 } else { ends[first - 1] };
        let copy = text[start..ends[end - 1]].trim_end_matches(' ');
        let mut job = egui::text::LayoutJob::single_section(copy.to_owned(), format.clone());
        job.break_on_newline = false;
        job.halign = egui::Align::Center;
        // Whole-point output rounding must not decide whether a word fits.
        job.round_output_to_gui = false;
        ui.painter().layout_job(job)
    };
    let minimum = (0..ends.len())
        .map(|first| shape(first, first + 1).size().x)
        .fold(0.0, f32::max);
    let width = available.max(minimum);
    let mut breaks = Vec::new();
    let mut first = 0;
    while first < ends.len() {
        let mut end = first + 1;
        while end < ends.len() && shape(first, end + 1).size().x <= width {
            end += 1;
        }
        breaks.push(end);
        first = end;
    }
    if breaks.len() == 1 {
        return natural;
    }
    // The pinned Chromium reference balances at most six lines. Longer copy
    // keeps greedy breaks and does not enter the paragraph-wide search.
    let lines = breaks.len();
    if lines <= 6 {
        let count = ends.len();
        let mut scores = vec![vec![(f32::INFINITY, 0); count + 1]; lines + 1];
        scores[0][0].0 = 0.0;
        for line in 1..=lines {
            for first in line - 1..count {
                let previous = scores[line - 1][first].0;
                if !previous.is_finite() {
                    continue;
                }
                for (end, candidate) in scores[line].iter_mut().enumerate().skip(first + 1) {
                    let remaining = width - shape(first, end).size().x;
                    if remaining < 0.0 {
                        break;
                    }
                    let score = previous + remaining * remaining;
                    if score < candidate.0 {
                        *candidate = (score, first);
                    }
                }
            }
        }
        // The greedy partition is always a feasible solution at this width.
        assert!(scores[lines][count].0.is_finite());
        let mut end = count;
        for line in (1..=lines).rev() {
            breaks[line - 1] = end;
            end = scores[line][end].1;
        }
    }

    // Keep source text and intrinsic metrics from the natural job; soft breaks
    // change drawing rows, not the source character map or accessible label.
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
        .expect("Empty sets an explicit line height");
    let mut widest = 0.0_f32;
    let mut first = 0;
    for end in breaks {
        let line = shape(first, end);
        let mut placed = line.rows[0].clone();
        if ends[end - 1] < text.len() {
            let row = Arc::make_mut(&mut placed.row);
            let mut glyph = space.expect("a soft break follows an ASCII space");
            glyph.pos.x = line.size().x;
            glyph.advance_width = 0.0;
            glyph.first_vertex = row.visuals.mesh.vertices.len() as u32;
            row.glyphs.push(glyph);
        }
        Arc::make_mut(&mut placed.row).size.y = line_height;
        let y = galley.rows.len() as f32 * line_height;
        placed.pos.y = (y * ui.pixels_per_point()).round() / ui.pixels_per_point();
        placed.ends_with_newline = false;
        galley.mesh_bounds |= placed.visuals.mesh_bounds.translate(placed.pos.to_vec2());
        galley.num_vertices += placed.visuals.mesh.vertices.len();
        galley.num_indices += placed.visuals.mesh.indices.len();
        widest = widest.max(line.size().x);
        galley.rows.push(placed);
        first = end;
    }
    galley.rect = egui::Rect::from_min_size(
        egui::pos2(-widest * 0.5, 0.0),
        egui::vec2(widest, galley.rows.len() as f32 * line_height),
    );
    Arc::new(galley)
}
