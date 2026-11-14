//! Inline Alert cases using the existing host's layout and paint phases.
use crate::input::{Layout, Phase};
use egui_prism::{
    alert::{Alert, AlertVariant},
    egui,
    theme::Theme,
};

pub struct AlertCase {
    icon: egui::TextureHandle,
}
impl AlertCase {
    pub fn new(context: &egui::Context) -> Self {
        let mut image = egui::ColorImage::filled([16, 16], egui::Color32::TRANSPARENT);
        for y in 2..14 {
            for x in 2..14 {
                if (7..9).contains(&x) || (7..9).contains(&y) {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            }
        }
        Self {
            icon: context.load_texture("alert-plus", image, egui::TextureOptions::LINEAR),
        }
    }
    pub fn show(&self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Alerts"));
        frame.label(
            ui,
            "Caller-owned inline notices. No activation, focus, timer or dismissal.",
        );
        ui.scope(|ui| {
            ui.set_max_width(448.0);
            for (variant, title, description) in [
                (
                    AlertVariant::Default,
                    "Heads up!",
                    "You can add components to your app using the CLI.",
                ),
                (
                    AlertVariant::Destructive,
                    "Unable to save",
                    "Your changes are still available in this window.",
                ),
            ] {
                draw(
                    Alert::new(theme, title)
                        .title(title)
                        .description(description)
                        .variant(variant)
                        .icon((self.icon.id(), egui::vec2(16.0, 16.0))),
                    ui,
                    frame,
                );
            }
            draw(
                Alert::new(theme, "title-only").title("A title without a description or icon"),
                ui,
                frame,
            );
            draw(
                Alert::new(theme, "description-only")
                    .description("A description without a title.")
                    .icon((self.icon.id(), egui::vec2(16.0, 16.0))),
                ui,
                frame,
            );
        });
        frame.label(
            ui,
            "160pt content: title elides, description wraps, full text remains accessible",
        );
        ui.scope(|ui| {
            ui.set_max_width(160.0);
            draw(Alert::new(theme,"narrow").title("A deliberately long title for a bounded notice").description("A long description wraps onto several lines inside the available width and remains accessible."),ui,frame);
        });
        frame.label(
            ui,
            "160pt legal breaks; long content overflows to the parent clip",
        );
        ui.scope(|ui| {
            ui.set_max_width(160.0);
            for (id, text) in [
                (
                    "hyphen-wrap",
                    "alpha-beta/gamma alpha-beta/gamma alpha-beta/gamma",
                ),
                (
                    "url-wrap",
                    "https://example.invalid/alpha-beta/gamma?query=value",
                ),
                ("word-overflow", "Supercalifragilisticexpialidocious"),
            ] {
                draw(Alert::new(theme, id).description(text), ui, frame);
            }
        });
        frame.label(ui, "320pt notice clipped to 54pt height");
        ui.scope(|ui| {
            ui.set_max_width(320.0);
            let clip=egui::Rect::from_min_size(ui.cursor().min,egui::vec2(320.0,54.0)).intersect(ui.clip_rect()); ui.set_clip_rect(clip);
            draw(Alert::new(theme,"clipped").title("Visible title").description("This longer description continues beyond the visible parent clip. It remains complete in accessibility metadata."),ui,frame);
        });
        frame.label(ui, "Parent opacity 60% over a tinted surface");
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(52, 120, 246))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_max_width(320.0);
                ui.multiply_opacity(0.6);
                draw(
                    Alert::new(theme, "opacity")
                        .title("Shared opacity")
                        .description("Text, border and background composite together.")
                        .variant(AlertVariant::Destructive),
                    ui,
                    frame,
                );
            });
    }
}
fn draw(alert: Alert<'_, '_>, ui: &mut egui::Ui, frame: &mut Layout) {
    let output = alert.show(ui);
    frame.track(match frame.phase {
        Phase::Register => output.response,
        Phase::Paint => output.paint(ui.painter()).expect("alert compositor"),
    });
}
