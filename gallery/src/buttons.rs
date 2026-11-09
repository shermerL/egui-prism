use crate::input::{self, Draw, Layout};
use egui_prism::{
    button::{Button, ButtonRole, ButtonSize, IconButton},
    egui,
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Invalid,
    Reversed,
    Preview {
        role: ButtonRole,
        size: ButtonSize,
        kind: u8,
    },
    Instance(usize),
    PreviewCount,
    Count(usize),
}
pub(super) struct ButtonCase {
    icon: egui::TextureHandle,
    disabled: bool,
    invalid: bool,
    reversed: bool,
    preview_activations: u64,
    counts: [u64; 2],
}
impl ButtonCase {
    pub(super) fn new(context: &egui::Context) -> Self {
        let mut image = egui::ColorImage::filled([16, 16], egui::Color32::TRANSPARENT);
        for y in 2..14 {
            for x in 2..14 {
                if (7..9).contains(&x) || (7..9).contains(&y) {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            }
        }
        Self {
            icon: context.load_texture("gallery-plus", image, egui::TextureOptions::LINEAR),
            disabled: false,
            invalid: false,
            reversed: false,
            preview_activations: 0,
            counts: [0; 2],
        }
    }
    pub(super) fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable buttons").into(),
            Control::Invalid => ui.checkbox(&mut self.invalid, "Invalid buttons").into(),
            Control::Reversed => ui
                .checkbox(&mut self.reversed, "Reverse button instances")
                .into(),
            Control::PreviewCount => ui
                .label(format!("Preview activations: {}", self.preview_activations))
                .into(),
            Control::Count(i) => ui.label(format!("Activations: {}", self.counts[i])).into(),
            Control::Instance(i) => {
                if self.disabled {
                    ui.disable();
                }
                Button::new(
                    theme,
                    format!("Activate {}", if i == 0 { "A" } else { "B" }),
                )
                .register(ui)
                .into()
            }
            Control::Preview { role, size, kind } => {
                if self.disabled {
                    ui.disable();
                }
                let image = egui::Image::new((self.icon.id(), egui::vec2(16., 16.)));
                if kind == 2 || kind == 3 {
                    let image = if kind == 3 {
                        image.tint(egui::Color32::RED)
                    } else {
                        image
                    };
                    return IconButton::new(
                        theme,
                        image,
                        if kind == 3 {
                            "Explicit red icon"
                        } else {
                            "Add item"
                        },
                    )
                    .size(size)
                    .invalid(self.invalid)
                    .register(ui)
                    .into();
                }
                if kind == 4 {
                    ui.style_mut()
                        .text_styles
                        .insert(egui::TextStyle::Button, egui::FontId::proportional(24.));
                }
                let label = match kind {
                    1 => "Add item",
                    4 => "Host button font",
                    5 => "A deliberately long action label that keeps its intrinsic width",
                    6 => "Clipped parent preview",
                    _ => match role {
                        ButtonRole::Primary => "Primary",
                        ButtonRole::Secondary => "Secondary",
                        ButtonRole::Outline => "Outline",
                        ButtonRole::Ghost => "Ghost",
                        ButtonRole::Destructive => "Destructive",
                        ButtonRole::Link => "Link",
                    },
                };
                let mut button = Button::new(theme, label)
                    .role(role)
                    .size(size)
                    .invalid(self.invalid);
                if kind == 1 || kind == 6 {
                    button = button.icon(image);
                }
                button.register(ui).into()
            }
        }
    }
    pub(super) fn activated(&mut self, control: Control) {
        match control {
            Control::Instance(i) => self.counts[i] += 1,
            Control::Preview { .. } => self.preview_activations += 1,
            _ => {}
        }
    }
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Prism buttons"));
        frame.label(ui,"Hover, press, or use Tab and Enter/Space. Theme and scale changes preserve action counts.");
        ui.horizontal_wrapped(|ui| {
            for c in [Control::Disabled, Control::Invalid] {
                self.widget(ui, theme, frame, c);
            }
        });
        frame.label(ui, "Roles and sizes: XS / Small / Default / Large");
        for role in [
            ButtonRole::Primary,
            ButtonRole::Secondary,
            ButtonRole::Outline,
            ButtonRole::Ghost,
            ButtonRole::Destructive,
            ButtonRole::Link,
        ] {
            ui.horizontal_wrapped(|ui| {
                for size in [
                    ButtonSize::ExtraSmall,
                    ButtonSize::Small,
                    ButtonSize::Medium,
                    ButtonSize::Large,
                ] {
                    self.widget(
                        ui,
                        theme,
                        frame,
                        Control::Preview {
                            role,
                            size,
                            kind: 0,
                        },
                    );
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            self.widget(
                ui,
                theme,
                frame,
                Control::Preview {
                    role: ButtonRole::Primary,
                    size: ButtonSize::Medium,
                    kind: 1,
                },
            );
            for size in [
                ButtonSize::ExtraSmall,
                ButtonSize::Small,
                ButtonSize::Medium,
                ButtonSize::Large,
            ] {
                self.widget(
                    ui,
                    theme,
                    frame,
                    Control::Preview {
                        role: ButtonRole::Primary,
                        size,
                        kind: 2,
                    },
                );
            }
        });
        ui.horizontal_wrapped(|ui| {
            self.widget(
                ui,
                theme,
                frame,
                Control::Preview {
                    role: ButtonRole::Primary,
                    size: ButtonSize::Medium,
                    kind: 3,
                },
            );
            frame.label(ui, "Explicit image tint");
        });
        self.widget(
            ui,
            theme,
            frame,
            Control::Preview {
                role: ButtonRole::Primary,
                size: ButtonSize::Medium,
                kind: 4,
            },
        );
        frame.scroll(ui, true, |ui, frame| {
            self.widget(
                ui,
                theme,
                frame,
                Control::Preview {
                    role: ButtonRole::Outline,
                    size: ButtonSize::Medium,
                    kind: 5,
                },
            );
        });
        frame.label(ui, "Parent opacity 60% with a clipped image and label");
        ui.scope(|ui| {
            ui.set_opacity(0.6);
            ui.set_clip_rect(ui.clip_rect().intersect(egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(160., 36.),
            )));
            self.widget(
                ui,
                theme,
                frame,
                Control::Preview {
                    role: ButtonRole::Outline,
                    size: ButtonSize::Medium,
                    kind: 6,
                },
            );
        });
        self.widget(ui, theme, frame, Control::PreviewCount);
        ui.separator();
        frame.label(ui, "Independent instances");
        self.widget(ui, theme, frame, Control::Reversed);
        for i in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("button-instance", i), |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.widget(ui, theme, frame, Control::Instance(i));
                    self.widget(ui, theme, frame, Control::Count(i));
                });
            });
        }
        frame.label(ui,egui::RichText::new("Icon names are exposed through egui metadata; actual screen-reader support is not verified.").small());
    }
    fn widget(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout, c: Control) {
        frame.widget(ui, input::Control::Button(c), |ui| {
            self.control(ui, theme, c)
        });
    }
}
