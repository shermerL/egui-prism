//! Public Card composition with an explicit headless recording compositor.
use egui_prism::{
    button::Button,
    card::Card,
    compositing::{self, Group},
    egui,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct Host(Mutex<Vec<Group>>);
impl compositing::Backend for Host {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}
fn main() -> Result<(), compositing::Error> {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    let host = Arc::new(Host::default());
    let _installation = compositing::install(&ctx, host.clone())?;
    let theme = Theme::light();
    let mut result = Ok(());
    ctx.run_ui(Default::default(), |ui| {
        theme.apply(ui.style_mut());
        ui.set_max_width(320.);
        result = Card::new(&theme)
            .show(ui, |card| {
                card.header("Project", Some("Caller-owned content and actions."));
                card.content(|body| {
                    body.ui()
                        .label("Ordinary egui content is painted separately.");
                });
                card.footer(36.0, |footer| {
                    // An ordered host retains this ButtonOutput and routes events to handle().
                    // draw() opts its painting into the Card's group without capturing callbacks.
                    let button = Button::new(&theme, "Save").register(footer.ui());
                    if let Some(group) = button.drawing {
                        footer.draw(group);
                    }
                });
            })
            .paint(ui.painter())
            .map(|_| ());
    })
    .drop_without_applying_deltas();
    result?;
    assert_eq!(host.0.lock().unwrap().len(), 1);
    Ok(())
}
