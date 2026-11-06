use egui_prism::{
    button::{Button, ButtonRole, ButtonSize, IconButton},
    egui,
    theme::Theme,
};

pub(super) struct ButtonCase {
    icon: egui::TextureHandle,
    disabled: bool,
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
            reversed: false,
            preview_activations: 0,
            counts: [0; 2],
        }
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism buttons");
        ui.label("Hover, press, or use Tab and Enter/Space. Theme and scale changes preserve action counts.");
        ui.checkbox(&mut self.disabled, "Disable buttons");
        ui.label("Roles and sizes");
        for (name, role) in [
            ("Primary", ButtonRole::Primary),
            ("Secondary", ButtonRole::Secondary),
            ("Outline", ButtonRole::Outline),
            ("Ghost", ButtonRole::Ghost),
            ("Destructive", ButtonRole::Destructive),
        ] {
            ui.horizontal_wrapped(|ui| {
                for size in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
                    if ui
                        .add_enabled(
                            !self.disabled,
                            Button::new(theme, name).role(role).size(size),
                        )
                        .clicked()
                    {
                        self.preview_activations += 1;
                    }
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.disabled,
                    Button::new(theme, "Add item").icon((self.icon.id(), egui::vec2(16.0, 16.0))),
                )
                .clicked()
            {
                self.preview_activations += 1;
            }
            for size in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
                if ui
                    .add_enabled(
                        !self.disabled,
                        IconButton::new(
                            theme,
                            (self.icon.id(), egui::vec2(16.0, 16.0)),
                            "Add item",
                        )
                        .size(size),
                    )
                    .clicked()
                {
                    self.preview_activations += 1;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            let red_icon =
                egui::Image::new((self.icon.id(), egui::vec2(16.0, 16.0))).tint(egui::Color32::RED);
            if ui
                .add_enabled(
                    !self.disabled,
                    IconButton::new(theme, red_icon, "Explicit red icon"),
                )
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
            if ui
                .add_enabled(!self.disabled, Button::new(theme, "Host button font"))
                .clicked()
            {
                self.preview_activations += 1;
            }
        });
        if ui.add_enabled(!self.disabled, Button::new(theme, "A deliberately long action label that wraps within the available content width").role(ButtonRole::Outline)).clicked() {
            self.preview_activations += 1;
        }
        ui.label(format!("Preview activations: {}", self.preview_activations));
        ui.separator();
        ui.label("Independent instances");
        ui.checkbox(&mut self.reversed, "Reverse button instances");
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            let id = ui.make_persistent_id(("button-instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            !self.disabled,
                            Button::new(
                                theme,
                                format!("Activate {}", if index == 0 { "A" } else { "B" }),
                            ),
                        )
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
