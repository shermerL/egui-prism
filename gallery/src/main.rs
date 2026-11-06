//! Minimal native and browser host for the component library.

use egui_prism::{egui, theme::Theme};

struct Gallery {
    dark: bool,
    sample_enabled: bool,
    sample_text: String,
}

impl Gallery {
    fn new(context: &egui::Context) -> Self {
        let gallery = Self {
            dark: false,
            sample_enabled: true,
            sample_text: "Select this text, then switch themes.".into(),
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("egui-prism");
                    ui.label("Semantic themes · Engineering preview");
                    let old_dark = self.dark;
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.dark, false, "Light");
                        ui.selectable_value(&mut self.dark, true, "Dark");
                    });
                    if old_dark != self.dark {
                        self.apply_theme(ui.ctx());
                        ui.ctx().request_repaint();
                    }
                    ui.separator();
                    ui.label("Paired surfaces and foregrounds");
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
                    let card_width = pairs.iter().map(|(name, pair)| {
                        ui.painter().layout_no_wrap((*name).into(), font.clone(), pair.foreground).size().x
                    }).fold(0.0_f32, f32::max)
                        + theme.metrics.container_margin.sum().x + 2.0;
                    let gap = theme.metrics.item_spacing.x;
                    let columns = (((ui.available_width() + gap) / (card_width + gap)) as usize).clamp(1, 4);
                    egui::Grid::new("semantic-palette").num_columns(columns).show(ui, |ui| {
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
                    ui.separator();
                    ui.heading("Harness controls (egui)");
                    ui.label("Theme switching preserves the values below. Prism widgets are planned.");
                    ui.checkbox(&mut self.sample_enabled, "Sample state");
                    let label = ui.label("Sample text");
                    ui.add(egui::TextEdit::singleline(&mut self.sample_text).desired_width(f32::INFINITY))
                        .labelled_by(label.id);
                    ui.horizontal(|ui| {
                        let _ = ui.selectable_label(true, "Selected foreground");
                        let _ = ui.add_enabled(false, egui::Button::new("egui disabled opacity"));
                    });
                    ui.small("Disabled token swatches are opaque pairs. Standard egui disabled controls use opacity.");
                    ui.separator();
                    ui.small("Renderer: glow / OpenGL on native, WebGL 2 on the web");
                    ui.hyperlink_to("Source", "https://github.com/shermerL/egui-prism");
                });
            });
    }
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
