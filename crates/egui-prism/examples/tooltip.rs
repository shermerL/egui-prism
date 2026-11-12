//! Headless host integration; the collector is not a GPU renderer.
use egui_prism::{
    compositing::{self, Group},
    egui,
    theme::Theme,
    tooltip::{Tooltip, TooltipRegistration},
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Collector(Mutex<Vec<Group>>);
impl compositing::Backend for Collector {
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
    context.options_mut(|options| options.ordered_input = true);
    let collector = Arc::new(Collector::default());
    let _installation = compositing::install(&context, collector.clone())?;
    let theme = Theme::light();
    let id = egui::Id::unique("save-action");
    let rect = egui::Rect::from_min_size(egui::pos2(40.0, 100.0), egui::vec2(120.0, 36.0));
    for time in [0.0, 0.2] {
        let mut result = Ok(());
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                time: Some(time),
                events: if time == 0.0 {
                    vec![egui::Event::PointerMoved(rect.center())]
                } else {
                    vec![]
                },
                ..Default::default()
            },
            |ui| {
                context.begin_input();
                let painter = ui.painter().clone();
                let trigger = ui.interact(rect, id, egui::Sense::click());
                trigger.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Save"));
                // The trigger, not Tooltip, owns its keyboard focus policy.
                context.memory_mut(|memory| {
                    memory.register_focus_policy(
                        id,
                        trigger.layer_id,
                        egui::EventFilter {
                            escape: true,
                            ..Default::default()
                        },
                    )
                });
                let mut hints = [
                    Tooltip::new(&theme, &trigger, "Save changes to this document").register(ui),
                ];
                while let Some(event) = context.next_input_event() {
                    TooltipRegistration::handle_all(&mut hints, &event);
                    if event.target == Some(id) && context.read_response(id).unwrap().clicked() {
                        println!("Save activated by the ordinary trigger");
                    }
                    // Dispatch other targets and actual ancestor scrolling here.
                }
                painter.rect_filled(rect, 8, theme.colors.secondary.background);
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Save",
                    egui::FontId::proportional(14.0),
                    theme.colors.secondary.foreground,
                );
                assert!(hints[0].output().open);
                result = hints[0].paint(&painter).map(|_| ());
            },
        );
        output.drop_without_applying_deltas();
        result?;
    }
    assert!(!collector.0.lock().unwrap().is_empty());
    Ok(())
}
