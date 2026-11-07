//! Native acceptance gallery for the component library.

mod native;

mod buttons;
mod checkboxes;
mod compositing;
mod interaction;
mod text_inputs;

use egui_prism::{egui, theme::Theme};
use interaction::InteractionCase;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Theme,
    Buttons,
    Inputs,
    Checkboxes,
    Interaction,
    Layout,
    Compositing,
}

struct Gallery {
    compositor: egui_prism_wgpu::Host,
    compositing: compositing::CompositingCase,
    dark: bool,
    case: Case,
    narrow: bool,
    interaction: InteractionCase,
    buttons: buttons::ButtonCase,
    inputs: text_inputs::InputCase,
    checkboxes: checkboxes::CheckboxCase,
}

impl Gallery {
    fn new(
        context: &egui::Context,
        device: &wgpu::Device,
    ) -> Result<Self, egui_prism::compositing::Error> {
        let compositor = egui_prism_wgpu::Host::new(context, device)?;
        let automatic = std::env::args().any(|arg| arg == "--compositing-check");
        egui_prism::theme::install_fonts(context, egui::FontDefinitions::default());
        let gallery = Self {
            compositor,
            compositing: compositing::CompositingCase::new(context, automatic),
            dark: false,
            case: if automatic {
                Case::Compositing
            } else {
                Case::Theme
            },
            narrow: false,
            interaction: InteractionCase::default(),
            buttons: buttons::ButtonCase::new(context),
            inputs: text_inputs::InputCase::default(),
            checkboxes: checkboxes::CheckboxCase::default(),
        };
        gallery.apply_theme(context);
        Ok(gallery)
    }

    fn theme(&self) -> Theme {
        if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn apply_theme(&self, context: &egui::Context) {
        let theme = self.theme();
        context.set_theme(if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        context.all_styles_mut(|style| theme.apply(style));
    }
}

impl Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, device: &wgpu::Device) {
        self.compositor.check().expect("compositing host failure");
        if self.compositing.take_restart_request() {
            let rect = egui::Rect::from_min_size(egui::pos2(4.0, 4.0), egui::vec2(8.0, 8.0));
            let painter = egui::Painter::new(
                ui.ctx().clone(),
                egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::unique("revoked-compositor"),
                ),
                rect,
            );
            let group = egui_prism::compositing::Group::new(
                rect,
                1.0,
                vec![egui_prism::compositing::Item::Shape(
                    egui::epaint::ClippedShape::new(
                        rect,
                        egui::Shape::rect_filled(rect, 0, egui::Color32::RED),
                    ),
                )],
            )
            .expect("stale callback probe");
            egui_prism::compositing::paint(&painter, group).expect("old host");
            self.compositor.invalidate();
            self.compositor =
                egui_prism_wgpu::Host::new(ui.ctx(), device).expect("restart compositor");
        }
        let theme = self.theme();
        if self.compositing.automatic() {
            egui::CentralPanel::default().show(ui, |ui| self.compositing.show(ui, &theme));
            return;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(theme.metrics.page_margin))
            .show(ui, |ui| {
                ui.heading("egui-prism");
                ui.label("Parity correction in progress: reference tokens and Geist fonts; components await individual acceptance.");
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.case, Case::Theme, "Theme");
                    ui.selectable_value(&mut self.case, Case::Compositing, "Compositing");
                    ui.selectable_value(&mut self.case, Case::Buttons, "Buttons");
                    ui.selectable_value(&mut self.case, Case::Inputs, "Inputs");
                    ui.selectable_value(&mut self.case, Case::Checkboxes, "Checkboxes");
                    ui.selectable_value(&mut self.case, Case::Interaction, "Interaction");
                    ui.selectable_value(&mut self.case, Case::Layout, "Layout");
                });
                let old_dark = self.dark;
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.dark, false, "Light");
                    ui.selectable_value(&mut self.dark, true, "Dark");
                    ui.checkbox(&mut self.narrow, "320pt content");
                    ui.label("Scale:");
                    for (factor, label) in [(1.0, "100%"), (1.5, "150%")] {
                        if ui.selectable_label(ui.ctx().zoom_factor() == factor, label).clicked() {
                            ui.ctx().set_zoom_factor(factor);
                        }
                    }
                });
                if self.dark != old_dark {
                    self.apply_theme(ui.ctx());
                    ui.ctx().request_repaint();
                }
                ui.separator();
                egui::ScrollArea::vertical().id_salt("gallery-content").show(ui, |ui| {
                    if self.narrow { ui.set_max_width(ui.available_width().min(320.0)); }
                    match self.case {
                        Case::Compositing => self.compositing.show(ui, &theme),
                        Case::Theme => {
                            ui.heading("Semantic palette");
                            show_palette(ui, &theme);
                            let _ = ui.selectable_label(true, "Selected foreground");
                            ui.label("Opaque disabled tokens are paired colors. Standard egui disabled controls use opacity.");
                        }
                        Case::Buttons => self.buttons.show(ui, &theme),
                        Case::Inputs => self.inputs.show(ui, &theme),
                        Case::Checkboxes => self.checkboxes.show(ui, &theme),
                        Case::Interaction | Case::Layout => {
                            ui.heading("Harness controls (egui)");
                            ui.label("Activate with the pointer or Tab and Enter. Counts change once per activation and remain still while idle.");
                            ui.horizontal_wrapped(|ui| {
                                ui.checkbox(&mut self.interaction.disabled, "Disable actions");
                                ui.checkbox(&mut self.interaction.reversed, "Reverse instances");
                            });
                            if self.case == Case::Layout {
                                ui.label("Long labels use the same values and stable IDs as the interaction case. Try narrow content and 150% scale, then scroll to the second instance.");
                            }
                            let responses = self.interaction.show(ui, self.case == Case::Layout);
                            if responses.iter().any(egui::Response::has_focus) {
                                ui.small("An action has keyboard focus.");
                            }
                        }
                    }
                    ui.separator();
                    ui.small(format!("Development v{} · {} · {}", env!("CARGO_PKG_VERSION"),
                        if cfg!(target_os = "macos") { "wgpu Metal" } else if cfg!(target_os = "windows") { "wgpu DX12" } else { "wgpu Vulkan" },
                        if cfg!(debug_assertions) { "debug" } else { "release" }));
                    ui.small(format!("Build: {}", option_env!("EGUI_PRISM_BUILD_ID").unwrap_or("local development")));
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to("Source", "https://github.com/shermerL/egui-prism");
                        ui.hyperlink_to("MIT license", "https://github.com/shermerL/egui-prism/blob/main/LICENSE");
                        ui.hyperlink_to("Third-party notices", "https://github.com/shermerL/egui-prism/blob/main/THIRD_PARTY_NOTICES.md");
                    });
                });
            });
    }
}

fn show_palette(ui: &mut egui::Ui, theme: &Theme) {
    let pairs = [
        ("Page", theme.colors.background),
        ("Surface", theme.colors.surface),
        ("Muted", theme.colors.muted),
        ("Primary", theme.colors.primary),
        ("Selected", theme.colors.selection),
        ("Destructive", theme.colors.destructive),
        ("Disabled token", theme.colors.disabled),
        ("Focus / pressed", theme.colors.focus),
    ];
    let font = egui::TextStyle::Body.resolve(ui.style());
    let card_width = pairs
        .iter()
        .map(|(name, pair)| {
            ui.painter()
                .layout_no_wrap((*name).into(), font.clone(), pair.foreground)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max)
        + theme.metrics.container_margin.sum().x
        + 2.0;
    let gap = theme.metrics.item_spacing.x;
    let columns = (((ui.available_width() + gap) / (card_width + gap)) as usize).clamp(1, 4);
    egui::Grid::new("semantic-palette")
        .num_columns(columns)
        .show(ui, |ui| {
            for (index, (name, pair)) in pairs.into_iter().enumerate() {
                egui::Frame::new()
                    .fill(pair.background)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.metrics.corner_radius)
                    .inner_margin(theme.metrics.container_margin)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(name).color(pair.foreground))
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    });
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}
