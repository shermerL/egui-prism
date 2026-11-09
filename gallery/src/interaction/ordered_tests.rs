use super::*;
use crate::input::{Control as RoutedControl, Phase};
use egui_prism::theme::Theme;

fn layout(context: &egui::Context, case: &mut InteractionCase, width: f32) -> Layout {
    let mut ui = egui::Ui::new(
        context.clone(),
        egui::Id::unique("ordered-interaction-test"),
        egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 1000.0),
        )),
    );
    let mut layout = Layout::new(Phase::Register);
    case.show_ordered(&mut ui, true, &mut layout);
    layout
}

fn action_rect(layout: &Layout) -> egui::Rect {
    layout
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.control,
                RoutedControl::Interaction(Control::Action(0))
            )
        })
        .unwrap()
        .response
        .rect
}

#[test]
fn ordered_tab_text_and_click_use_geometry_refreshed_after_edit() {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    context.all_styles_mut(|style| Theme::light().apply(style));
    let mut case = InteractionCase::default();
    let mut narrow_action = egui::Rect::NOTHING;

    // Measure the destination layout without manufacturing response IDs.
    context
        .run_ui(egui::RawInput::default(), |_| {
            context.begin_input();
            let registered = layout(&context, &mut case, 280.0);
            narrow_action = action_rect(&registered);
            while context.next_input_event().is_some() {}
            let _ = layout(&context, &mut case, 280.0);
        })
        .drop_without_applying_deltas();

    let position = narrow_action.center();
    let mut width = 640.0;
    let mut refreshed = false;
    context
        .run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 1000.0),
                )),
                events: vec![
                    egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::Text("ordered edit".into()),
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |_| {
                context.begin_input();
                let mut registered = layout(&context, &mut case, width);
                assert_ne!(action_rect(&registered), narrow_action);

                while let Some(event) = context.next_input_event() {
                    let handled = event.target.and_then(|id| {
                        let entry = registered
                            .entries
                            .iter()
                            .find(|entry| entry.response.id == id)?;
                        let RoutedControl::Interaction(control) = entry.control else {
                            return None;
                        };
                        let response = case
                            .control(&mut entry.open(&context), control, true)
                            .response;
                        Some((control, response))
                    });
                    if handled.is_some_and(|(control, response)| {
                        matches!(control, Control::Text(0)) && response.changed()
                    }) {
                        width = 280.0;
                        context.input_layout(|| {
                            let replacement = layout(&context, &mut case, width);
                            registered.retire(&context, &replacement);
                            registered = replacement;
                        });
                        assert_eq!(action_rect(&registered), narrow_action);
                        refreshed = true;
                    }
                }
                let _ = layout(&context, &mut case, width);
            },
        )
        .drop_without_applying_deltas();

    assert!(refreshed);
    assert_eq!(case.instances[0].text, "ordered edit");
    assert!(case.instances[1].text.is_empty());
    assert_eq!(case.instances[0].activations, 1);
    assert_eq!(case.instances[1].activations, 0);
}
