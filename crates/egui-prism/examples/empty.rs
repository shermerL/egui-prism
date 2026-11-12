//! A headless caller switches states and routes actions; no network or media object.
use egui_prism::{
    button::{Button, ButtonOutput},
    compositing::{self, Group},
    egui,
    empty::Empty,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Empty,
    Loading,
    Failed,
}
struct Demo {
    state: State,
    attempts: usize,
}
#[derive(Default)]
struct Host(Mutex<Vec<Group>>);
impl compositing::Backend for Host {
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
fn present(
    ui: &mut egui::Ui,
    theme: &Theme,
    state: State,
    paint: bool,
) -> Result<Option<ButtonOutput>, compositing::Error> {
    match state {
        State::Empty => {
            let output = Empty::new(theme, "projects")
                .title("No projects yet")
                .description("Create your first project to get started.")
                .show(ui, |content| {
                    let button = Button::new(theme, "Create project").register(content.ui());
                    if let Some(drawing) = &button.drawing {
                        content.draw(drawing.clone());
                    }
                    button
                });
            let button = if paint {
                output.paint(ui.painter())?.inner
            } else {
                output.inner
            };
            Ok(Some(button))
        }
        State::Loading => {
            ui.label("Loading projects... Completion and cancellation belong to the caller.");
            Ok(None)
        }
        State::Failed => {
            ui.colored_label(
                theme.colors.destructive.background,
                "Projects could not be loaded.",
            );
            let button = Button::new(theme, "Retry").register(ui);
            if paint && let Some(drawing) = &button.drawing {
                compositing::paint(ui.painter(), drawing.clone())?;
            }
            Ok(Some(button))
        }
    }
}
fn main() -> Result<(), compositing::Error> {
    let ctx = egui::Context::default();
    ctx.options_mut(|o| o.ordered_input = true);
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    let host = Arc::new(Host::default());
    let _installation = compositing::install(&ctx, host.clone())?;
    let theme = Theme::light();
    let mut demo = Demo {
        state: State::Empty,
        attempts: 0,
    };
    // External outcomes are supplied by the caller, not inferred by Empty.
    for (state, activate) in [
        (State::Empty, false),
        (State::Empty, true),
        (State::Loading, false),
        (State::Failed, true),
        (State::Empty, false),
    ] {
        demo.state = state;
        let mut result = Ok(());
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640., 480.),
                )),
                events: if activate {
                    vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }]
                } else {
                    vec![]
                },
                ..Default::default()
            },
            |ui| {
                theme.apply(ui.style_mut());
                ctx.begin_input();
                let rect = ui.available_rect_before_wrap();
                let id = ui.scope_id().with("demo");
                let builder = || egui::UiBuilder::new().scope_id(id).max_rect(rect);
                let mut action = ctx
                    .input_layout(|| {
                        present(&mut ui.new_child(builder()), &theme, demo.state, false)
                    })
                    .unwrap();
                if activate && let Some(action) = &action {
                    action.response.request_focus();
                }
                while let Some(event) = ctx.next_input_event() {
                    if let Some(button) = &action {
                        button.cancel_unfocused();
                        if event.target == Some(button.response.id)
                            && button.handle(&event).clicked()
                        {
                            demo.attempts += 1;
                            demo.state = State::Loading;
                            // Retire the disappeared control before reading another event.
                            ctx.remove_input_widget(button.response.id);
                            action = ctx
                                .input_layout(|| {
                                    present(&mut ui.new_child(builder()), &theme, demo.state, false)
                                })
                                .unwrap();
                        }
                    }
                }
                result =
                    present(&mut ui.new_child(builder()), &theme, demo.state, true).map(|_| ());
            },
        )
        .drop_without_applying_deltas();
        result?;
        if activate {
            assert_eq!(demo.state, State::Loading);
        }
    }
    assert_eq!(demo.attempts, 2);
    assert!(!host.0.lock().unwrap().is_empty());
    Ok(())
}
