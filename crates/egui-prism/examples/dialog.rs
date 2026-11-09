//! Headless ordered-input integration; a native host installs its real compositor.
use std::sync::{Arc, Mutex};

use egui_prism::{
    button::{Button, ButtonOutput},
    compositing::{self, Group},
    dialog::{Dialog, DialogUi},
    egui,
    theme::Theme,
};

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

fn body(slot: &mut DialogUi<'_>, theme: &Theme) -> ButtonOutput {
    let mut button = Button::new(theme, "Apply").register(slot.ui());
    slot.focusable(&button.response);
    if let Some(drawing) = button.drawing.take() {
        slot.draw(drawing);
    }
    button
}

fn main() -> Result<(), compositing::Error> {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    let backend = Arc::new(CollectingHost::default());
    let _installation = compositing::install(&context, backend.clone())?;
    let theme = Theme::light();
    let mut open = true;
    let mut applied = 0;
    let mut painted = Ok(());
    let events = [egui::Key::Enter, egui::Key::Escape]
        .into_iter()
        .flat_map(|key| {
            [true, false].map(|pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
        })
        .collect();
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            let context = ui.ctx();
            context.begin_input();
            // Register background controls first. This example starts already open.
            let mut dialog = Dialog::new(&theme, egui::Id::unique("settings"), "Settings")
                .description("The application decides when to accept a close request.")
                .register(context, open, |slot| body(slot, &theme));
            while let Some(event) = context.next_input_event() {
                if let Some(button) = &dialog.inner {
                    button.cancel_unfocused();
                    if event.target == Some(button.response.id) && button.handle(&event).clicked() {
                        applied += 1;
                    }
                }
                let intent = dialog.handle(&event);
                if intent.close_requested.is_some() {
                    open = false;
                }
                if intent.layout_changed || intent.close_requested.is_some() {
                    dialog.refresh(open, |slot| body(slot, &theme));
                }
                // Only now dispatch this event to background controls by its target.
            }
            painted = dialog.paint(|slot| body(slot, &theme));
        },
    );
    output.drop_without_applying_deltas();
    painted?;
    assert_eq!(applied, 1);
    assert!(!open);
    // No window, GPU execution, or platform acceptance is implied by this example.
    Ok(())
}
