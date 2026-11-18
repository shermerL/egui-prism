//! Targeted CSS normal-wrapping contract, not native/browser visual acceptance.
use egui_prism::{
    card::Card,
    compositing::Item,
    egui,
    theme::{self, Theme},
};

#[test]
fn card_title_does_not_split_an_unbreakable_word() {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    ctx.run_ui(Default::default(), |ui| {
        let theme = Theme::light();
        theme.apply(ui.style_mut());
        ui.set_max_width(120.0);
        let output = Card::new(&theme).show(ui, |card| {
            card.header("Antidisestablishmentarianism", None);
        });
        let drawing = output.drawing.as_ref().unwrap();
        let title = drawing
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Shape(shape) => match &shape.shape {
                    egui::Shape::Text(text) => Some(text),
                    _ => None,
                },
                _ => None,
            })
            .unwrap();
        assert_eq!(
            title.galley.rows.len(),
            1,
            "CSS normal wrapping preserves this word"
        );
    })
    .drop_without_applying_deltas();
}

#[test]
fn title_and_description_share_the_widest_min_content_track() {
    for (title, description) in [
        ("Antidisestablishmentarianism", "Saved projects"),
        ("Saved projects", "Antidisestablishmentarianism"),
        ("one\u{a0}two\u{a0}three\u{a0}four", "Saved projects"),
    ] {
        let ctx = egui::Context::default();
        theme::install_fonts(&ctx, egui::FontDefinitions::default());
        ctx.run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            theme.apply(ui.style_mut());
            ui.set_max_width(120.0);
            let output = Card::new(&theme).show(ui, |card| {
                card.header(title, Some(description));
            });
            assert_eq!(output.response.rect.width(), 120.0);
            let text: Vec<_> = output
                .drawing
                .as_ref()
                .unwrap()
                .items()
                .iter()
                .filter_map(|item| match item {
                    Item::Shape(shape) => match &shape.shape {
                        egui::Shape::Text(text) => Some(text),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            assert_eq!(text.len(), 2);
            for (text, expected) in text.iter().zip([title, description]) {
                assert_eq!(text.galley.text(), expected);
                assert_eq!(text.galley.rows.len(), 1);
                assert!(!text.galley.elided);
            }
            assert_eq!(text[0].pos.x, text[1].pos.x);
            assert_eq!(text[1].pos.y - text[0].pos.y, 24.0);
        })
        .drop_without_applying_deltas();
    }
}
