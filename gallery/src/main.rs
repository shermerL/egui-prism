//! Minimal native and browser host for the component library.

use egui_prism::egui;

#[derive(Default)]
struct Gallery {
    sample_enabled: bool,
    sample_text: String,
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("egui-prism");
            ui.label("Engineering preview");
            ui.label("The workspace is ready for component development.");
            ui.separator();
            ui.heading("Harness controls (egui)");
            ui.label("These controls verify the host. Prism components and themes are planned.");
            ui.checkbox(&mut self.sample_enabled, "Sample state");
            ui.horizontal(|ui| {
                let label = ui.label("Sample text");
                ui.text_edit_singleline(&mut self.sample_text)
                    .labelled_by(label.id);
            });
            ui.separator();
            ui.label("Renderer: glow / OpenGL on native, WebGL 2 on the web");
            ui.hyperlink_to("Source", "https://github.com/shermerL/egui-prism");
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
        Box::new(|_| Ok(Box::<Gallery>::default())),
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
                Box::new(|_| Ok(Box::<Gallery>::default())),
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
