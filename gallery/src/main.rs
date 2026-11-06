//! Shared native and browser acceptance gallery for the component library.

mod buttons;
mod interaction;

use egui_prism::{egui, theme::Theme};
use interaction::InteractionCase;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Theme,
    Buttons,
    Interaction,
    Layout,
}

struct Gallery {
    dark: bool,
    case: Case,
    narrow: bool,
    interaction: InteractionCase,
    buttons: buttons::ButtonCase,
}

impl Gallery {
    fn new(context: &egui::Context) -> Self {
        let gallery = Self {
            dark: false,
            case: Case::Theme,
            narrow: false,
            interaction: InteractionCase::default(),
            buttons: buttons::ButtonCase::new(context),
        };
        gallery.apply_theme(context);
        gallery
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

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = self.theme();
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(theme.metrics.page_margin))
            .show(ui, |ui| {
                ui.heading("egui-prism");
                ui.label("Available: themes and Prism buttons. Other widgets are planned.");
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.case, Case::Theme, "Theme");
                    ui.selectable_value(&mut self.case, Case::Buttons, "Buttons");
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
                        Case::Theme => {
                            ui.heading("Semantic palette");
                            show_palette(ui, &theme);
                            let _ = ui.selectable_label(true, "Selected foreground");
                            ui.label("Opaque disabled tokens are paired colors. Standard egui disabled controls use opacity.");
                        }
                        Case::Buttons => self.buttons.show(ui, &theme),
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
                        if cfg!(target_arch = "wasm32") { "WebGL 2" } else { "native OpenGL" },
                        if cfg!(debug_assertions) { "debug" } else { "release" }));
                    ui.small(format!("Build: {}", option_env!("EGUI_PRISM_BUILD_ID").unwrap_or("local development")));
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to("Source", "https://github.com/shermerL/egui-prism");
                        ui.hyperlink_to("MIT license", if cfg!(target_arch = "wasm32") {
                            "/egui-prism/LICENSE"
                        } else { "https://github.com/shermerL/egui-prism/blob/main/LICENSE" });
                        ui.hyperlink_to("Third-party notices", if cfg!(target_arch = "wasm32") {
                            "/egui-prism/THIRD_PARTY_NOTICES.md"
                        } else { "https://github.com/shermerL/egui-prism/blob/main/THIRD_PARTY_NOTICES.md" });
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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-prism gallery",
        options,
        Box::new(|cc| Ok(Box::new(Gallery::new(&cc.egui_ctx)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast as _;

        let document = web_sys::window()
            .expect("The gallery requires a browser window")
            .document()
            .expect("The gallery requires a document");
        let canvas = document
            .get_element_by_id("gallery")
            .expect("The gallery canvas is part of index.html")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("The gallery element must be a canvas");
        let runner = eframe::WebRunner::new();
        let result = runner
            .start(
                canvas,
                eframe::WebOptions {
                    renderer: eframe::Renderer::Glow,
                    webgl_context_option: eframe::WebGlContextOption::WebGl2,
                    ..Default::default()
                },
                Box::new(|cc| Ok(Box::new(Gallery::new(&cc.egui_ctx)))),
            )
            .await;
        match result {
            Ok(()) => {
                if let Some(status) = document.get_element_by_id("status") {
                    status.remove();
                }
            }
            Err(error) => {
                runner.destroy();
                web_sys::console::error_1(&error);
                if let Some(status) = document.get_element_by_id("status") {
                    status.set_text_content(Some(
                        "The gallery could not start. WebGL 2 is required. See the browser console for details.",
                    ));
                }
            }
        }
    });
}
