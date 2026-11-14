//! Controlled description-only probe, not a browser/native/AT parity test.
//! Run with --nocapture and retain ALERT_* TSV records with the tested SHA.
use egui_prism::{
    alert::Alert,
    compositing::Item,
    egui,
    theme::{self, Theme},
};

fn hex(text: &str) -> String {
    text.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn wrapping_corpus_retains_complete_text_and_reports_rows() {
    let corpus = include_str!("../../../reference/fixtures/alert-wrapping.tsv");
    let cases: Vec<_> = corpus
        .lines()
        .map(|line| line.split_once('\t').unwrap())
        .collect();
    assert_eq!(cases.len(), 7);
    for ppp in [1.0, 1.5] {
        let context = egui::Context::default();
        theme::install_fonts(&context, egui::FontDefinitions::default());
        context.set_pixels_per_point(ppp);
        for width in [448.0, 320.0, 160.0, 24.0] {
            for (id, text) in &cases {
                let mut frame = context.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(800.0, 1200.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        let theme = Theme::light();
                        theme.apply(ui.style_mut());
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(32.0, 32.0),
                            egui::vec2(width, 1000.0),
                        );
                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                        let output = Alert::new(&theme, (id, width as u32))
                            .description(text)
                            .show(&mut child);
                        assert_eq!(output.response.rect.width(), width.max(34.0));
                        let group = output.drawing.as_ref().expect("visible alert");
                        let text_shapes: Vec<_> = group
                            .items()
                            .iter()
                            .filter_map(|item| match item {
                                Item::Shape(shape) => match &shape.shape {
                                    egui::Shape::Text(text) => Some((shape.clip_rect, text)),
                                    _ => None,
                                },
                                _ => None,
                            })
                            .collect();
                        assert_eq!(text_shapes.len(), 1);
                        let (clip, shape) = text_shapes[0];
                        assert_eq!(shape.galley.job.text, *text);
                        let rebuilt: String = shape
                            .galley
                            .rows
                            .iter()
                            .flat_map(|row| row.glyphs.iter().map(|glyph| glyph.chr))
                            .collect();
                        // Layout can discard line-break spaces, but not visible text
                        // or NBSP. This invariant does not certify CSS word boundaries.
                        assert_eq!(rebuilt.replace(' ', ""), text.replace(' ', ""));
                        println!(
                            "ALERT_CASE\t{id}\t{width}\t{ppp}\t{}\t{}\t{}\t{}",
                            output.response.rect.height(),
                            clip.width(),
                            shape.galley.rows.len(),
                            hex(text)
                        );
                        for (index, row) in shape.galley.rows.iter().enumerate() {
                            let text: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                            println!(
                                "ALERT_ROW\t{id}\t{width}\t{ppp}\t{index}\t{}\t{}",
                                row.size.x,
                                hex(&text)
                            );
                        }
                    },
                );
                assert_eq!(frame.pixels_per_point, ppp);
                frame.textures_delta.clear();
            }
        }
    }
}
