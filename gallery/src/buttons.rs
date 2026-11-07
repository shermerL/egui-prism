use egui_prism::{
    button::{Button, ButtonRole, ButtonSize, IconButton},
    egui,
    theme::Theme,
};

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
        // A local geometric mask keeps the example independent of fonts and loaders.
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

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism buttons");
        ui.label("Hover, press, or use Tab and Enter/Space. Theme and scale changes preserve action counts.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable buttons");
            ui.checkbox(&mut self.invalid, "Invalid buttons");
        });
        ui.label("Roles and sizes: XS / Small / Default / Large");
        for (name, role) in [
            ("Primary", ButtonRole::Primary),
            ("Secondary", ButtonRole::Secondary),
            ("Outline", ButtonRole::Outline),
            ("Ghost", ButtonRole::Ghost),
            ("Destructive", ButtonRole::Destructive),
            ("Link", ButtonRole::Link),
        ] {
            ui.horizontal_wrapped(|ui| {
                for size in [
                    ButtonSize::ExtraSmall,
                    ButtonSize::Small,
                    ButtonSize::Medium,
                    ButtonSize::Large,
                ] {
                    if show_enabled(ui, !self.disabled, |ui| {
                        Button::new(theme, name)
                            .role(role)
                            .size(size)
                            .invalid(self.invalid)
                            .show(ui)
                    })
                    .clicked()
                    {
                        self.preview_activations += 1;
                    }
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            if show_enabled(ui, !self.disabled, |ui| {
                Button::new(theme, "Add item")
                    .icon((self.icon.id(), egui::vec2(16.0, 16.0)))
                    .show(ui)
            })
            .clicked()
            {
                self.preview_activations += 1;
            }
            for size in [
                ButtonSize::ExtraSmall,
                ButtonSize::Small,
                ButtonSize::Medium,
                ButtonSize::Large,
            ] {
                if show_enabled(ui, !self.disabled, |ui| {
                    IconButton::new(theme, (self.icon.id(), egui::vec2(16.0, 16.0)), "Add item")
                        .size(size)
                        .invalid(self.invalid)
                        .show(ui)
                })
                .clicked()
                {
                    self.preview_activations += 1;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            let red_icon =
                egui::Image::new((self.icon.id(), egui::vec2(16.0, 16.0))).tint(egui::Color32::RED);
            if show_enabled(ui, !self.disabled, |ui| {
                IconButton::new(theme, red_icon, "Explicit red icon").show(ui)
            })
            .clicked()
            {
                self.preview_activations += 1;
            }
            ui.label("Explicit image tint");
        });
        ui.scope(|ui| {
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(24.0));
            if show_enabled(ui, !self.disabled, |ui| {
                Button::new(theme, "Host button font").show(ui)
            })
            .clicked()
            {
                self.preview_activations += 1;
            }
        });
        egui::ScrollArea::horizontal()
            .id_salt("long-button")
            .show(ui, |ui| {
                if show_enabled(ui, !self.disabled, |ui| {
                    Button::new(
                        theme,
                        "A deliberately long action label that keeps its intrinsic width",
                    )
                    .role(ButtonRole::Outline)
                    .show(ui)
                })
                .clicked()
                {
                    self.preview_activations += 1;
                }
            });
        ui.label("Parent opacity 60% with a clipped image and label");
        ui.scope(|ui| {
            ui.set_opacity(0.6);
            let clip = ui.clip_rect().intersect(egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(160.0, 36.0),
            ));
            ui.set_clip_rect(clip);
            if show_enabled(ui, !self.disabled, |ui| {
                Button::new(theme, "Clipped parent preview")
                    .icon((self.icon.id(), egui::vec2(16.0, 16.0)))
                    .role(ButtonRole::Outline)
                    .show(ui)
            })
            .clicked()
            {
                self.preview_activations += 1;
            }
        });
        ui.label(format!("Preview activations: {}", self.preview_activations));
        ui.separator();
        ui.label("Independent instances");
        ui.checkbox(&mut self.reversed, "Reverse button instances");
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            let id = ui.make_persistent_id(("button-instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if show_enabled(ui, !self.disabled, |ui| {
                        Button::new(
                            theme,
                            format!("Activate {}", if index == 0 { "A" } else { "B" }),
                        )
                        .show(ui)
                    })
                    .clicked()
                    {
                        self.counts[index] += 1;
                    }
                    ui.label(format!("Activations: {}", self.counts[index]));
                });
            });
        }
        ui.small("Icon names are exposed through egui metadata; actual screen-reader support is not verified.");
    }
}

fn show_enabled(
    ui: &mut egui::Ui,
    enabled: bool,
    show: impl FnOnce(&mut egui::Ui) -> egui_prism::button::ButtonOutput,
) -> egui::Response {
    let painter = ui.painter().clone();
    ui.add_enabled_ui(enabled, show)
        .inner
        .paint(&painter)
        .expect("button compositor")
}
