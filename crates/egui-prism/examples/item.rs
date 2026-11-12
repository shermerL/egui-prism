//! Headless Item submission; the collector is not a native window or GPU renderer.
use egui_prism::{
    button::{Button, ButtonRole},
    compositing::{self, Group},
    egui,
    item::{Item, ItemAction, ItemVariant},
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
    let theme = Theme::light();
    egui_prism::theme::install_fonts(&context, egui::FontDefinitions::default());
    context.all_styles_mut(|style| theme.apply(style));
    let mut submitted = Ok(());
    context
        .run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                ..Default::default()
            },
            |ui| {
                let output = Item::new(&theme, "project")
                    .title("Example project")
                    .description("The application owns the content and both independent actions.")
                    .variant(ItemVariant::Outline)
                    .action(ItemAction::new(
                        "open",
                        Button::new(&theme, "Open").role(ButtonRole::Outline),
                    ))
                    .action(ItemAction::new("archive", Button::new(&theme, "Archive")))
                    .show(ui);
                assert!(!output.response.sense.is_focusable());
                assert_eq!(output.actions.len(), 2);
                assert_ne!(output.actions[0].response.id, output.actions[1].response.id);
                assert!(
                    !output.actions[0]
                        .response
                        .rect
                        .intersects(output.actions[1].response.rect)
                );
                // Inspect each action's response here. Application side effects belong
                // here, not inside sizing/layout or the passive Item itself.
                submitted = output.paint(ui.painter()).map(|_| ());
            },
        )
        .drop_without_applying_deltas();
    submitted?;
    assert_eq!(*collector.0.lock().unwrap(), 1);
    Ok(())
}
