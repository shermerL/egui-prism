//! Separator cases using the host's existing layout and paint phases.

use crate::input::{Layout, Phase};
use egui_prism::{
    egui,
    separator::{Orientation, Separator},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
    frame.track(ui.heading("Separators"));
    frame.label(ui, "One-point borders; no activation or keyboard focus.");
    ui.scope(|ui| {
        ui.set_max_width(320.0);
        for decorative in [true, false] {
            frame.label(ui, if decorative { "Decorative" } else { "Semantic" });
            draw(Separator::new(theme).decorative(decorative), ui, frame);
            ui.allocate_ui_with_layout(
                egui::vec2(320.0, 48.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    frame.label(ui, "Left");
                    draw(
                        Separator::new(theme)
                            .orientation(Orientation::Vertical)
                            .decorative(decorative),
                        ui,
                        frame,
                    );
                    frame.label(ui, "Right");
                },
            );
        }
        frame.label(ui, "320pt separator clipped to 160pt");
        ui.scope(|ui| {
            let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(160.0, 8.0));
            ui.set_clip_rect(ui.clip_rect().intersect(clip));
            draw(Separator::new(theme), ui, frame);
        });
        frame.label(ui, "Parent opacity 60% over a tinted surface");
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(52, 120, 246))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(296.0);
                ui.multiply_opacity(0.6);
                draw(Separator::new(theme), ui, frame);
            });
    });
}

fn draw(separator: Separator<'_>, ui: &mut egui::Ui, frame: &mut Layout) {
    let output = separator.show(ui);
    frame.track(match frame.phase {
        Phase::Register => output.response,
        Phase::Paint => output.paint(ui.painter()).expect("separator compositor"),
    });
}
