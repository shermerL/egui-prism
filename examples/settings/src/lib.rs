//! Caller-owned settings and an explicit ordered-input integration.
//!
//! This recipe uses stock egui Modal, not Prism Dialog. Its controlled input
//! tests do not establish OS input, IME, screen-reader or framebuffer acceptance.

use egui_prism::{
    button::Button,
    compositing,
    egui::{self, Context, Event, Id, InputTrigger, Key, Response, Ui, UiBuilder},
    text_input::TextInput,
    theme::Theme,
};

/// Values and modal edit state owned by the application, not by a widget.
pub struct SettingsApp {
    /// The current setting, retained across redraws and modal cancellation.
    pub name: String,
    /// Independent application actions, demonstrating unrelated redraws.
    pub unrelated_actions: u64,
    /// Actions from the fixed-source third-party button.
    pub third_party_actions: u64,
    draft: Option<String>,
    return_focus: Option<Id>,
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            name: "Example".into(),
            unrelated_actions: 0,
            third_party_actions: 0,
            draft: None,
            return_focus: None,
        }
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Register,
    Event(Id),
    Paint,
}

impl Phase {
    fn paints(self) -> bool {
        matches!(self, Self::Paint)
    }
}

struct Scene {
    editor: Response,
    open: Response,
    unrelated: Response,
    third_party: Response,
    modal: Option<ModalScene>,
}

struct ModalScene {
    editor: Response,
    confirm: Response,
    cancel: Response,
    layer: egui::LayerId,
}

fn editor_id() -> Id {
    Id::unique("settings.name")
}

fn draft_id() -> Id {
    Id::unique("settings.draft")
}

fn modal_id() -> Id {
    Id::unique("settings.modal")
}

// Reconstruct only this recipe's local layout. Non-target widgets are registered
// under input_layout, so a third-party widget cannot consume another target's
// event. No arbitrary application closure is replayed and no action runs here.
fn invoke<T>(
    context: &Context,
    phase: Phase,
    previous: Option<&Response>,
    show: impl FnOnce() -> T,
) -> T {
    if matches!(phase, Phase::Event(target) if previous.is_none_or(|r| r.id != target)) {
        context.input_layout(show)
    } else {
        show()
    }
}

fn child(ui: &mut Ui, name: &'static str, offset: egui::Vec2, size: egui::Vec2) -> Ui {
    let rect = egui::Rect::from_min_size(ui.max_rect().min + offset, size);
    ui.new_child(
        UiBuilder::new()
            .scope_id(Id::unique(("settings.slot", name)))
            .max_rect(rect),
    )
}

impl SettingsApp {
    /// Install the selected fonts and input protocol before the first UI pass.
    pub fn configure_context(context: &Context) {
        shadcn_egui::Theme::vega().apply(context);
        context.options_mut(|options| options.ordered_input = true);
    }

    /// Process one ordered viewport pass and paint its final application state.
    pub fn ui(&mut self, ui: &mut Ui) -> Result<(), compositing::Error> {
        self.frame(ui, true).map(|_| ())
    }

    fn frame(&mut self, ui: &mut Ui, paint: bool) -> Result<Scene, compositing::Error> {
        let context = ui.ctx().clone();
        context.begin_input();
        let mut scene = self.scene(ui, Phase::Register, None)?;
        while let Some(routed) = context.next_input_event() {
            // Escape cancels only the open settings edit. It never commits a
            // draft, and layer revocation happens before another event routes.
            let escape = matches!(
                routed.trigger,
                InputTrigger::Event {
                    event: Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                        ..
                    },
                    ..
                }
            );
            if escape && self.draft.is_some() {
                self.close(
                    &context,
                    scene.modal.as_ref().expect("registered modal").layer,
                    false,
                );
            } else if let Some(target) = routed.target {
                let handled = self.scene(ui, Phase::Event(target), Some(&scene))?;
                if target == scene.open.id && handled.open.clicked() {
                    self.draft = Some(self.name.clone());
                    self.return_focus = Some(scene.open.id);
                    // Register the new modal before assigning its editor focus.
                    context.input_layout(|| self.scene(ui, Phase::Register, None))?;
                    context.memory_mut(|memory| memory.request_focus(draft_id()));
                } else if target == scene.unrelated.id && handled.unrelated.clicked() {
                    self.unrelated_actions += 1;
                } else if target == scene.third_party.id && handled.third_party.clicked() {
                    self.third_party_actions += 1;
                } else if let (Some(previous), Some(current)) = (&scene.modal, &handled.modal) {
                    if target == previous.confirm.id && current.confirm.clicked() {
                        self.close(&context, previous.layer, true);
                    } else if target == previous.cancel.id && current.cancel.clicked() {
                        self.close(&context, previous.layer, false);
                    }
                }
            }
            // Editor content and modal visibility can change geometry. Renew it
            // before routing the next event, without spending another frame.
            scene = context.input_layout(|| self.scene(ui, Phase::Register, None))?;
        }
        if paint {
            self.scene(ui, Phase::Paint, Some(&scene))
        } else {
            Ok(scene)
        }
    }

    fn close(&mut self, context: &Context, layer: egui::LayerId, confirm: bool) {
        let draft = self.draft.take().expect("an open edit owns its draft");
        if confirm {
            self.name = draft;
        }
        context.remove_input_layer(layer);
        context.memory_mut(|memory| {
            memory.request_focus(self.return_focus.take().expect("modal opener focus"));
        });
    }

    fn scene(
        &mut self,
        ui: &mut Ui,
        phase: Phase,
        previous: Option<&Scene>,
    ) -> Result<Scene, compositing::Error> {
        let context = ui.ctx().clone();
        let theme = Theme::light();
        let editor = invoke(&context, phase, previous.map(|p| &p.editor), || {
            let mut slot = child(ui, "name", egui::vec2(20., 20.), egui::vec2(320., 80.));
            let output = TextInput::new(&theme, &mut self.name, "Name")
                .id(editor_id())
                .show(&mut slot);
            if phase.paints() {
                output.paint(slot.painter())
            } else {
                Ok(output.response)
            }
        })?;
        let open = invoke(&context, phase, previous.map(|p| &p.open), || {
            let mut slot = child(ui, "open", egui::vec2(20., 120.), egui::vec2(320., 40.));
            let output = Button::new(&theme, "Edit in modal").show(&mut slot);
            if phase.paints() {
                output.paint(slot.painter())
            } else {
                Ok(output.response)
            }
        })?;
        let unrelated = invoke(&context, phase, previous.map(|p| &p.unrelated), || {
            let mut slot = child(
                ui,
                "unrelated",
                egui::vec2(20., 180.),
                egui::vec2(320., 40.),
            );
            let response = slot.button("Unrelated redraw");
            context.memory_mut(|memory| {
                memory.register_focus_policy(
                    response.id,
                    response.layer_id,
                    egui::EventFilter::default(),
                );
            });
            response
        });
        let third_party = invoke(&context, phase, previous.map(|p| &p.third_party), || {
            let mut slot = child(
                ui,
                "third-party",
                egui::vec2(20., 240.),
                egui::vec2(320., 40.),
            );
            let response = slot.add(shadcn_egui::components::button::Button::new(
                "Third-party action",
            ));
            // This caller-owned bridge supplies current-pass policy to a widget
            // whose upstream implementation targets ordinary egui execution.
            context.memory_mut(|memory| {
                memory.register_focus_policy(
                    response.id,
                    response.layer_id,
                    egui::EventFilter::default(),
                );
            });
            response
        });
        if phase.paints() {
            ui.painter().text(
                ui.max_rect().min + egui::vec2(20., 300.),
                egui::Align2::LEFT_TOP,
                format!(
                    "Unrelated: {}   Third-party: {}",
                    self.unrelated_actions, self.third_party_actions
                ),
                egui::FontId::proportional(14.),
                egui::Color32::BLACK,
            );
        }
        let modal = if let Some(draft) = &mut self.draft {
            let content = |ui: &mut Ui| -> Result<ModalScene, compositing::Error> {
                ui.set_min_size(egui::vec2(360., 190.));
                let previous = previous.and_then(|p| p.modal.as_ref());
                let editor = invoke(&context, phase, previous.map(|p| &p.editor), || {
                    let mut slot = child(ui, "draft", egui::vec2(8., 8.), egui::vec2(320., 80.));
                    let output = TextInput::new(&theme, draft, "Draft name")
                        .id(draft_id())
                        .show(&mut slot);
                    if phase.paints() {
                        output.paint(slot.painter())
                    } else {
                        Ok(output.response)
                    }
                })?;
                let confirm = invoke(&context, phase, previous.map(|p| &p.confirm), || {
                    let mut slot =
                        child(ui, "confirm", egui::vec2(8., 110.), egui::vec2(150., 40.));
                    let output = Button::new(&theme, "Confirm").show(&mut slot);
                    if phase.paints() {
                        output.paint(slot.painter())
                    } else {
                        Ok(output.response)
                    }
                })?;
                let cancel = invoke(&context, phase, previous.map(|p| &p.cancel), || {
                    let mut slot =
                        child(ui, "cancel", egui::vec2(180., 110.), egui::vec2(150., 40.));
                    let response = slot.button("Cancel");
                    context.memory_mut(|memory| {
                        memory.register_focus_policy(
                            response.id,
                            response.layer_id,
                            egui::EventFilter::default(),
                        );
                    });
                    response
                });
                Ok(ModalScene {
                    editor,
                    confirm,
                    cancel,
                    layer: ui.layer_id(),
                })
            };
            let modal = egui::Modal::new(modal_id());
            let result = if matches!(phase, Phase::Register) {
                modal.register(&context, content)
            } else {
                modal.show(&context, content)
            };
            Some(result.inner?)
        } else {
            None
        };
        Ok(Scene {
            editor,
            open,
            unrelated,
            third_party,
            modal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Harness {
        context: Context,
        app: SettingsApp,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let context = Context::default();
            SettingsApp::configure_context(&context);
            context.options_mut(|options| options.max_passes = 1.try_into().unwrap());
            Self {
                context,
                app: SettingsApp::default(),
                time: 0.,
            }
        }

        fn frame(&mut self, events: Vec<Event>) -> Scene {
            self.time += 1. / 60.;
            let mut scene = None;
            self.context
                .run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(800., 600.),
                        )),
                        events,
                        time: Some(self.time),
                        ..Default::default()
                    },
                    |ui| {
                        // Same update path as the native recipe; final GPU submission
                        // belongs to its separate host test, not this CPU evidence.
                        scene = Some(self.app.frame(ui, false).expect("stateful recipe"));
                    },
                )
                .drop_without_applying_deltas();
            scene.expect("one UI pass")
        }
    }

    fn click(response: &Response) -> Vec<Event> {
        let pos = response.rect.center();
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }

    fn key(key: Key, command: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command,
                ..Default::default()
            },
        }
    }

    fn replace(response: &Response, text: &str) -> Vec<Event> {
        let mut events = click(response);
        events.extend([key(Key::A, true), Event::Text(text.into())]);
        events
    }

    #[test]
    fn editing_survives_redraw_and_modal_confirm_cancel() {
        let mut h = Harness::new();
        let first = h.frame(vec![]);
        let scene = h.frame(replace(&first.editor, "First"));
        assert_eq!(h.app.name, "First");
        let scene = h.frame(click(&scene.unrelated));
        assert_eq!(h.app.unrelated_actions, 1);
        assert_eq!(h.app.name, "First");
        let scene = h.frame(replace(&scene.editor, "Second"));
        assert_eq!(h.app.name, "Second");
        let scene = h.frame(click(&scene.open));
        let modal = scene.modal.expect("opened modal");
        let scene = h.frame(replace(&modal.editor, "Discard"));
        assert_eq!(h.app.name, "Second");
        let scene = h.frame(click(&scene.modal.as_ref().unwrap().cancel));
        assert!(scene.modal.is_none());
        assert_eq!(h.app.name, "Second");
        assert_eq!(h.context.memory(|m| m.focused()), Some(scene.open.id));
        let scene = h.frame(click(&scene.open));
        let scene = h.frame(replace(&scene.modal.as_ref().unwrap().editor, "Committed"));
        let scene = h.frame(click(&scene.modal.as_ref().unwrap().confirm));
        assert!(scene.modal.is_none());
        assert_eq!(h.app.name, "Committed");
        assert_eq!(h.context.memory(|m| m.focused()), Some(scene.open.id));
        h.frame(replace(&scene.editor, "Still editable"));
        assert_eq!(h.app.name, "Still editable");
    }

    #[test]
    fn mixed_widgets_act_once_and_modal_blocks_background() {
        let mut h = Harness::new();
        let scene = h.frame(vec![]);
        let mut events = click(&scene.third_party);
        events.extend(click(&scene.unrelated));
        let scene = h.frame(events);
        assert_eq!((h.app.third_party_actions, h.app.unrelated_actions), (1, 1));
        h.frame(click(&scene.third_party));
        assert_eq!(h.app.third_party_actions, 2);
        let scene = h.frame(vec![key(Key::Enter, false)]);
        assert_eq!(h.app.third_party_actions, 3);
        let scene = h.frame(click(&scene.open));
        let mut events = click(&scene.third_party);
        events.extend(click(&scene.unrelated));
        let scene = h.frame(events);
        assert!(scene.modal.is_some());
        assert_eq!((h.app.third_party_actions, h.app.unrelated_actions), (3, 1));
        let scene = h.frame(vec![key(Key::Escape, false)]);
        assert!(scene.modal.is_none());
        assert_eq!(h.context.memory(|m| m.focused()), Some(scene.open.id));
        h.frame(vec![]);
        h.frame(vec![]);
        assert_eq!((h.app.third_party_actions, h.app.unrelated_actions), (3, 1));
    }
}
