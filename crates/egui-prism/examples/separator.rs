//! Compile the public API with caller-bounded horizontal and vertical separators.

use egui_prism::{
    egui,
    separator::{Orientation, Separator},
    theme::Theme,
};

fn main() {
    let context = egui::Context::default();
    context
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            ui.set_max_width(320.0);
            let horizontal = Separator::new(&theme).show(ui);
            assert_eq!(horizontal.response.rect.size(), egui::vec2(320.0, 1.0));
            ui.allocate_ui_with_layout(
                egui::vec2(320.0, 48.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let vertical = Separator::new(&theme)
                        .orientation(Orientation::Vertical)
                        .decorative(false)
                        .show(ui);
                    assert_eq!(vertical.response.rect.size(), egui::vec2(1.0, 48.0));
                    // A rendering host submits these frame-local drawings using
                    // output.paint(ui.painter()) or a shared parent Group.
                    assert!(vertical.drawing.is_some());
                },
            );
            assert!(horizontal.drawing.is_some());
        })
        .drop_without_applying_deltas();
}
