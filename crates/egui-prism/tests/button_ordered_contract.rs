//! Button behavior through the real ordered input cursor and a single UI pass.
use egui_prism::{button::Button, egui, theme::Theme};

fn key(key: egui::Key, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    }
}

fn run(events: Vec<egui::Event>) -> ([usize; 2], Option<usize>) {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    let theme = Theme::light();
    let mut counts = [0; 2];
    let mut focused = None;
    context
        .run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                context.begin_input();
                let controls = [
                    Button::new(&theme, "First").register(ui),
                    Button::new(&theme, "Second").register(ui),
                ];
                let mut previous = context.memory(|memory| memory.focused());
                while let Some(event) = context.next_input_event() {
                    if let Some(index) = controls
                        .iter()
                        .position(|button| Some(button.response.id) == event.target)
                        && controls[index].handle(&event).clicked()
                    {
                        counts[index] += 1;
                    }
                    let next = context.memory(|memory| memory.focused());
                    if previous != next
                        && let Some(old) = controls
                            .iter()
                            .find(|button| Some(button.response.id) == previous)
                    {
                        old.cancel_unfocused();
                    }
                    previous = next;
                }
                focused = controls
                    .iter()
                    .position(|button| Some(button.response.id) == previous);
            },
        )
        .drop_without_applying_deltas();
    (counts, focused)
}

#[test]
fn first_focus_keeps_arrows_and_escape_before_enter() {
    use egui::{Key, Modifiers};
    let mut events = Vec::new();
    for key_code in [Key::Tab, Key::ArrowDown, Key::Escape, Key::Enter] {
        events.push(key(key_code, true, Modifiers::NONE));
        events.push(key(key_code, false, Modifiers::NONE));
    }
    assert_eq!(run(events), ([1, 0], Some(0)));
}

#[test]
fn focus_transfer_cancels_space_even_when_focus_returns_in_the_same_pass() {
    use egui::{Key, Modifiers};
    let result = run(vec![
        key(Key::Tab, true, Modifiers::NONE),
        key(Key::Tab, false, Modifiers::NONE),
        key(Key::Space, true, Modifiers::NONE),
        key(Key::Tab, true, Modifiers::NONE),
        key(Key::Tab, false, Modifiers::NONE),
        key(Key::Tab, true, Modifiers::SHIFT),
        key(Key::Tab, false, Modifiers::SHIFT),
        key(Key::Space, false, Modifiers::NONE),
    ]);
    assert_eq!(result, ([0, 0], Some(0)));
}
