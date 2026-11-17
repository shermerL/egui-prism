//! Headless public API example; a native host supplies its GPU compositor.
use egui_prism::{
    alert::{Alert, AlertVariant},
    button::Button,
    compositing::{self, Group},
    egui,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct CollectingHost(Mutex<Vec<Group>>);
impl compositing::Backend for CollectingHost {
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
    let context = egui::Context::default();
    theme::install_fonts(&context, egui::FontDefinitions::default());
    let backend = Arc::new(CollectingHost::default());
    let _installation = compositing::install(&context, backend.clone())?;
    let theme = Theme::light();
    let mut result = Ok(());
    context
        .run_ui(Default::default(), |ui| {
            theme.apply(ui.style_mut());
            ui.set_max_width(320.0);
            result = (|| {
                Alert::new(&theme, "save-error")
                    .title("Unable to save")
                    .description("Your changes are still available in this window.")
                    .variant(AlertVariant::Destructive)
                    .show(ui)
                    .paint(ui.painter())?;
                // Application actions stay separate: an ordered host routes this Button
                // using ButtonOutput::handle and decides whether to retry a real operation.
                Button::new(&theme, "Try again")
                    .register(ui)
                    .paint(ui.painter())?;
                Ok(())
            })();
        })
        .drop_without_applying_deltas();
    result?;
    assert_eq!(backend.0.lock().unwrap().len(), 2);
    Ok(())
}
