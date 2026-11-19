//! Finite Item slots with independent caller-owned actions.
use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{
    button::{Button, ButtonRole, IconButton},
    egui,
    item::{Item, ItemAction, ItemSize, ItemVariant},
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Hidden,
    Reversed,
    Long,
    Small,
    Icon,
    Action(usize, usize),
    Count(usize),
}
pub(super) struct ItemCase {
    icon: egui::TextureHandle,
    disabled: bool,
    hidden: bool,
    reversed: bool,
    long: bool,
    small: bool,
    media: bool,
    counts: [[u64; 2]; 3],
}
impl ItemCase {
    pub(super) fn new(context: &egui::Context) -> Self {
        let mut image = egui::ColorImage::filled([16, 16], egui::Color32::TRANSPARENT);
        for y in 3..13 {
            for x in 3..13 {
                if (7..9).contains(&x) || (7..9).contains(&y) {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            }
        }
        Self {
            icon: context.load_texture("item-plus", image, egui::TextureOptions::LINEAR),
            disabled: false,
            hidden: false,
            reversed: false,
            long: false,
            small: false,
            media: true,
            counts: [[0; 2]; 3],
        }
    }
    pub(super) fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable actions").into(),
            Control::Hidden => ui.checkbox(&mut self.hidden, "Hide items").into(),
            Control::Reversed => ui.checkbox(&mut self.reversed, "Reverse items").into(),
            Control::Long => ui.checkbox(&mut self.long, "Long copy and action").into(),
            Control::Small => ui.checkbox(&mut self.small, "Small items").into(),
            Control::Icon => ui.checkbox(&mut self.media, "Leading icon").into(),
            Control::Count(i) => ui
                .label(format!(
                    "Item {i}: Open {}, Add {}",
                    self.counts[i][0], self.counts[i][1]
                ))
                .into(),
            Control::Action(_, _) => unreachable!("Item actions use their registered ButtonOutput"),
        }
    }
    pub(super) fn activated(&mut self, control: Control) {
        if let Control::Action(i, action) = control {
            self.counts[i][action] += 1;
        }
    }
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Items"));
        frame.label(ui, "Passive content and two independent actions. Counters survive reorder and page return.");
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Disabled,
                Control::Hidden,
                Control::Reversed,
                Control::Long,
                Control::Small,
                Control::Icon,
            ] {
                frame.widget(ui, input::Control::Item(control), |ui| {
                    self.control(ui, control)
                });
            }
        });
        if !self.hidden {
            for i in if self.reversed { [2, 1, 0] } else { [0, 1, 2] } {
                let variant = [
                    ItemVariant::Default,
                    ItemVariant::Outline,
                    ItemVariant::Muted,
                ][i];
                let title = ["Default item", "Outlined item", "Muted item"][i];
                let mut item = Item::new(theme, ("gallery-item", i))
                    .title(title)
                    .description(if self.long { "A deliberately long description demonstrates the supported text layout, keeps the complete accessible name, and must not steal the independent actions' hit regions." } else { "Caller-owned content and independent actions." })
                    .variant(variant)
                    .size(if self.small { ItemSize::Small } else { ItemSize::Default })
                    .action(ItemAction::new("open", Button::new(theme, if self.long { "Download original archive" } else { "Open" }).role(ButtonRole::Outline)).enabled(!self.disabled))
                    .action(ItemAction::icon("add", IconButton::new(theme, (self.icon.id(), egui::vec2(16.0, 16.0)), "Add item")).enabled(!self.disabled));
                if self.media {
                    item = item.icon(egui::Image::new((self.icon.id(), egui::vec2(16.0, 16.0))));
                }
                let mut output = item.register(ui);
                frame.ids.extend(output.ids.iter().copied());
                for (action, button) in std::mem::take(&mut output.actions).into_iter().enumerate()
                {
                    frame.item_action(Control::Action(i, action), button);
                }
                frame.track(match frame.phase {
                    Phase::Register => output.response,
                    Phase::Paint => output.paint(ui.painter()).expect("item compositor"),
                });
                frame.widget(ui, input::Control::Item(Control::Count(i)), |ui| {
                    self.control(ui, Control::Count(i))
                });
            }
        }
        frame.label(ui, "Use 320pt content, theme and scale controls. Browser raster parity and real assistive technology remain separate specialist checks.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_actions_route_once_and_survive_reorder_without_hidden_activation() {
        let context = egui::Context::default();
        context.options_mut(|options| options.ordered_input = true);
        egui_prism::theme::install_fonts(&context, egui::FontDefinitions::default());
        let theme = Theme::light();
        context.all_styles_mut(|style| theme.apply(style));
        let mut case = ItemCase::new(&context);
        let mut time = 0.0;
        let mut frame = |case: &mut ItemCase, events| {
            time += 0.02;
            let mut targets = Vec::new();
            context
                .run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(700.0, 900.0),
                        )),
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        context.begin_input();
                        let mut layout = Layout::new(Phase::Register);
                        case.show(ui, &theme, &mut layout);
                        for entry in &layout.entries {
                            if let input::Control::Item(Control::Action(i, action)) = entry.control
                            {
                                targets.push((i, action, entry.response.id, entry.response.rect));
                            }
                        }
                        while let Some(event) = context.next_input_event() {
                            if let Some(entry) = layout
                                .entries
                                .iter()
                                .find(|entry| Some(entry.response.id) == event.target)
                                && let Some(button) = &entry.button
                                && button.handle(&event).clicked()
                                && let input::Control::Item(control) = entry.control
                            {
                                case.activated(control);
                            }
                            layout.cancel_interactions();
                        }
                    },
                )
                .drop_without_applying_deltas();
            targets
        };
        let initial = frame(&mut case, vec![]);
        assert_eq!(initial.len(), 6);
        let target = initial
            .iter()
            .find(|(i, a, _, _)| (*i, *a) == (0, 0))
            .unwrap();
        let original_id = target.2;
        let position = target.3.center();
        let click = |position| {
            vec![
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
            ]
        };
        frame(&mut case, click(position));
        assert_eq!(case.counts[0], [1, 0]);
        frame(&mut case, vec![]);
        assert_eq!(case.counts[0], [1, 0]);
        case.reversed = true;
        let reversed = frame(&mut case, vec![]);
        let target = reversed
            .iter()
            .find(|(i, a, _, _)| (*i, *a) == (0, 0))
            .unwrap();
        assert_eq!(target.2, original_id);
        let moved = target.3.center();
        case.disabled = true;
        frame(&mut case, click(moved));
        assert_eq!(case.counts[0], [1, 0]);
        case.hidden = true;
        case.disabled = false;
        assert!(frame(&mut case, click(moved)).is_empty());
        assert_eq!(case.counts[0], [1, 0]);
        case.hidden = false;
        frame(&mut case, vec![]);
        assert_eq!(case.counts[0], [1, 0]);
    }
}
