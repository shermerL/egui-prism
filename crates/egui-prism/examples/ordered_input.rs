//! A host-owned callback with local scrolling and current-event geometry.
use egui_prism::egui::{self, Event, Id, InputTrigger, Key, Modifiers, Rect, Sense, Ui, UiBuilder};

#[derive(Clone, Copy)]
struct Geometry {
    rect: Rect,
    clip: Rect,
}

// This local shell declares content extent without traversing its controls.
fn scroll_shell(ui: &mut Ui) -> Geometry {
    let mut child = ui.new_child(UiBuilder::new().scope_id(Id::unique("viewport")).max_rect(
        Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200., 120.)),
    ));
    let out = egui::ScrollArea::vertical()
        .id_salt("scroll")
        .max_height(120.)
        .auto_shrink([false, false])
        .content_margin(0.)
        .show_viewport(&mut child, |ui, _| {
            ui.set_min_size(egui::vec2(180., 600.));
            ui.clip_rect()
        });
    let origin = out.inner_rect.min - out.state.unclamped_offset();
    Geometry {
        rect: Rect::from_min_size(origin + egui::vec2(20., 60.), egui::vec2(120., 36.)),
        clip: out.inner,
    }
}

fn control(ui: &mut Ui, geometry: Geometry) -> egui::Response {
    let mut child = ui.new_child(UiBuilder::new().scope_id(Id::unique("content")));
    child.set_clip_rect(geometry.clip);
    child.interact(geometry.rect, Id::unique("increment"), Sense::click())
}

fn main() {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    let mut count = 0;
    let pos = egui::pos2(70., 55.);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(200., 120.),
        )),
        events: vec![
            key(Key::Tab),
            key(Key::Enter),
            Event::PointerMoved(pos),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., -40.),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
                modifiers: Modifiers::NONE,
            },
            pointer(pos, true),
            pointer(pos, false),
        ],
        ..Default::default()
    };
    let output = context.run_ui(input, |ui| {
        context.begin_input();
        let mut geometry = scroll_shell(ui);
        control(ui, geometry);
        while let Some(event) = context.next_input_event() {
            if matches!(
                event.trigger,
                InputTrigger::Time
                    | InputTrigger::Event {
                        event: Event::MouseWheel { .. },
                        ..
                    }
            ) {
                geometry = scroll_shell(ui);
                context.input_layout(|| {
                    control(ui, geometry);
                });
            }
            if event.target == Some(Id::unique("increment")) && control(ui, geometry).clicked() {
                count += 1;
            }
        }
        geometry = scroll_shell(ui);
        let response = control(ui, geometry);
        let color = if response.hovered() {
            egui::Color32::LIGHT_BLUE
        } else {
            egui::Color32::LIGHT_GRAY
        };
        let painter = ui.painter().with_clip_rect(geometry.clip);
        painter.rect_filled(geometry.rect, 6.0, color);
        painter.text(
            geometry.rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("Count: {count}"),
            egui::FontId::proportional(14.0),
            egui::Color32::BLACK,
        );
    });
    assert_eq!(count, 2, "keyboard activation then a click after scrolling");
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
fn pointer(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
