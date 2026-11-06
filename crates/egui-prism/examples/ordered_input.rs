//! A minimal host-owned input callback; no native window is created here.
use egui_prism::egui::{self, Event, Id, Key, Modifiers, Rect, Sense};

fn main() {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    let mut count = 0;
    let input = egui::RawInput {
        events: vec![key(Key::Tab), key(Key::Enter)],
        ..Default::default()
    };
    let output = context.run_ui(input, |ui| {
        context.begin_input();
        let id = Id::unique("increment");
        let rect = Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(120.0, 36.0));
        ui.interact(rect, id, Sense::click());
        while let Some(event) = context.next_input_event() {
            if event.target == Some(id) && ui.interact(rect, id, Sense::click()).clicked() {
                count += 1;
            }
        }
        let response = ui.interact(rect, id, Sense::click());
        let color = if response.hovered() {
            egui::Color32::LIGHT_BLUE
        } else {
            egui::Color32::LIGHT_GRAY
        };
        ui.painter().rect_filled(rect, 6.0, color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("Count: {count}"),
            egui::FontId::proportional(14.0),
            egui::Color32::BLACK,
        );
    });
    assert_eq!(count, 1);
    output.drop_without_applying_deltas();
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
