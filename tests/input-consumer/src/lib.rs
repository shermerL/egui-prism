//! Compile-time identity checks at an independent application root.

/// The official platform bridge returns the same Context type as Prism.
pub fn platform_context(state: &egui_winit::State) -> &egui_prism::egui::Context {
    state.egui_ctx()
}

/// Official renderer input accepts primitives from the root egui Context.
pub fn renderer_input(
    renderer: &mut egui_wgpu::Renderer,
    device: &egui_wgpu::wgpu::Device,
    queue: &egui_wgpu::wgpu::Queue,
    encoder: &mut egui_wgpu::wgpu::CommandEncoder,
    ctx: &egui_prism::egui::Context,
    output: egui::FullOutput,
) {
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    let _ = renderer.update_buffers(
        device,
        queue,
        encoder,
        &primitives,
        &egui_wgpu::ScreenDescriptor {
            size_in_pixels: [640, 480],
            pixels_per_point: output.pixels_per_point,
        },
    );
}

/// The optional official provider implements the root font-provider trait.
#[cfg(feature = "system-fonts")]
pub fn font_provider() -> std::sync::Arc<dyn egui_prism::egui::FontProvider> {
    std::sync::Arc::new(egui_system_font_provider::SystemFontProvider::new())
}

#[test]
fn shared_context_exposes_ordered_input() {
    let context: egui_prism::egui::Context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    context
        .run_ui(egui::RawInput::default(), |_| {
            context.begin_input();
            assert!(context.next_input_event().is_none());
        })
        .drop_without_applying_deltas();
}
