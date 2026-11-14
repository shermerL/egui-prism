//! Headless host submission; this collector is not a desktop or GPU renderer.
use egui_prism::{
    compositing::{self, Group},
    egui,
    spinner::Spinner,
    theme::Theme,
};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct Collector(Mutex<usize>);
impl compositing::Backend for Collector {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        *self.0.lock().unwrap() += 1;
        Ok(egui::PaintCallback {
            rect: group.bounds(),
            callback: Arc::new(()),
        })
    }
}
fn main() -> Result<(), compositing::Error> {
    let context = egui::Context::default();
    let collector = Arc::new(Collector::default());
    let _installed = compositing::install(&context, collector.clone())?;
    for time in [0.0, 0.25] {
        let mut result = Ok(());
        context
            .run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    let painter = ui.painter().clone();
                    // The application decides busy state, label and reduced-motion policy.
                    result = Spinner::new(&Theme::light(), "load-projects", "Loading projects")
                        .animate(true)
                        .show(ui)
                        .paint(&painter)
                        .map(|_| ());
                },
            )
            .drop_without_applying_deltas();
        result?;
    }
    assert_eq!(*collector.0.lock().unwrap(), 2);
    Ok(())
}
