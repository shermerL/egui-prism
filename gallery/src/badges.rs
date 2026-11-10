//! Static Badge cases using the host's existing layout and paint phases.

use crate::input::{Layout, Phase};
use egui_prism::{
    badge::{Badge, BadgeVariant},
    egui,
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
    frame.track(ui.heading("Badges"));
    frame.label(ui, "Static text; no activation or focus behavior.");
    ui.horizontal_wrapped(|ui| {
        for (variant, text) in [
            (BadgeVariant::Primary, "Primary"),
            (BadgeVariant::Secondary, "Secondary"),
            (BadgeVariant::Destructive, "Destructive"),
            (BadgeVariant::Outline, "Outline"),
            (BadgeVariant::Ghost, "Ghost"),
            (BadgeVariant::Link, "Link"),
        ] {
            draw(Badge::new(theme, text).variant(variant), ui, frame);
        }
    });
    frame.label(ui, "Long text inside a 320pt horizontal scroll area");
    ui.scope(|ui| {
        ui.set_max_width(320.0);
        frame.scroll(ui, true, |ui, frame| {
            draw(Badge::new(theme, "A deliberately long badge label stays on one line and remains available by scrolling").variant(BadgeVariant::Outline), ui, frame);
        });
    });
    frame.label(ui, "Parent opacity 60% over a tinted surface");
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(52, 120, 246))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.multiply_opacity(0.6);
            ui.horizontal(|ui| {
                for (variant, text) in [
                    (BadgeVariant::Primary, "Primary"),
                    (BadgeVariant::Destructive, "Destructive"),
                    (BadgeVariant::Outline, "Outline"),
                ] {
                    draw(Badge::new(theme, text).variant(variant), ui, frame);
                }
            });
        });
}

fn draw(badge: Badge<'_>, ui: &mut egui::Ui, frame: &mut Layout) {
    let output = badge.show(ui);
    frame.track(match frame.phase {
        Phase::Register => output.response,
        Phase::Paint => output.paint(ui.painter()).expect("badge compositor"),
    });
}
