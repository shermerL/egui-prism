//! Minimal headless host integration.
//!
//! A native application uses this same register/event/paint ordering inside its
//! own viewport callback and installs its real compositor. This example collects
//! groups rather than creating a window or pretending to render GPU callbacks.

use std::sync::{Arc, Mutex};

use egui_prism::{
    compositing::{self, Group},
    egui,
    select::{Select, SelectItem, SelectSize},
    theme::Theme,
};

#[derive(Default)]
struct CollectingHost(Mutex<Vec<Group>>);

impl compositing::Backend for CollectingHost {
    fn callback(
        &self,
        _context: &egui::Context,
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

fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn main() -> Result<(), compositing::Error> {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);

    let backend = Arc::new(CollectingHost::default());
    let _installation = compositing::install(&context, backend.clone())?;
    let theme = Theme::light();
    let items = [
        SelectItem::new("apple", "Apple"),
        SelectItem::new("apricot", "Apricot").disabled(true),
        SelectItem::new("banana", "Banana"),
    ];
    let mut selected = None;
    let mut paint_result = Ok(());

    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 480.0),
            )),
            events: vec![
                key(egui::Key::Enter),
                key(egui::Key::ArrowDown),
                key(egui::Key::Enter),
            ],
            ..Default::default()
        },
        |ui| {
            context.begin_input();

            // Capture before any add_enabled_ui/disable scope.
            let painter = ui.painter().clone();
            let mut registration = Select::new(&theme, egui::Id::unique("fruit"), &items)
                .placeholder("Choose an option")
                .size(SelectSize::Default)
                .width(192.0)
                .invalid(false)
                .register(ui, &selected);

            // Initial focus is an explicit host decision in this example.
            registration.response.request_focus();

            while let Some(event) = context.next_input_event() {
                // Do not filter by event.target: popup releases, blur, and time
                // opportunities also belong to the select's input contract.
                let response = registration.handle(&event, &mut selected);
                if response.changed() {
                    println!("Selected: {selected:?}");
                }
            }

            paint_result = registration.paint(&selected, &painter).map(|_| ());
        },
    );
    output.drop_without_applying_deltas();
    paint_result?;

    assert_eq!(selected, Some("banana"));
    assert!(!backend.0.lock().unwrap().is_empty());
    Ok(())
}
