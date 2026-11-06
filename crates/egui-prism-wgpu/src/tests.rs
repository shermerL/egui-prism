//! Opt-in GPU contracts: run on a native adapter, not as CPU acceptance.
use super::*;
use egui_wgpu::{Callback, CallbackResources, CallbackTrait};
struct Producer {
    view: wgpu::TextureView,
    mode: u8,
}
impl Producer {
    fn write(&self, encoder: &mut wgpu::CommandEncoder) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("caller image producer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
}
impl CallbackTrait for Producer {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _: &wgpu::Queue,
        _: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        _: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if self.mode == 0 {
            self.write(encoder);
        }
        if self.mode == 1 {
            let mut own = device.create_command_encoder(&Default::default());
            self.write(&mut own);
            vec![own.finish()]
        } else {
            Vec::new()
        }
    }
    fn finish_prepare(
        &self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        _: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if self.mode == 2 {
            self.write(encoder);
        }
        Vec::new()
    }
    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        _: &mut wgpu::RenderPass<'static>,
        _: &CallbackResources,
    ) {
    }
}
fn screen() -> ScreenDescriptor {
    ScreenDescriptor {
        size_in_pixels: [32, 32],
        pixels_per_point: 1.0,
    }
}
fn jobs(
    context: &egui::Context,
    image: egui::TextureId,
) -> (Vec<egui::ClippedPrimitive>, egui::TexturesDelta) {
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(32.0, 32.0),
            )),
            ..Default::default()
        },
        |ui| {
            let rect = Rect::from_min_max(egui::pos2(4.0, 4.0), egui::pos2(28.0, 28.0));
            let mut image_mesh = Mesh::with_texture(image);
            image_mesh.add_rect_with_uv(
                Rect::from_min_max(egui::pos2(8.0, 8.0), egui::pos2(24.0, 24.0)),
                unit_uv(),
                egui::Color32::WHITE,
            );
            let group = Group::new(
                rect,
                0.5,
                vec![
                    Item::Shape(egui::epaint::ClippedShape::new(
                        rect,
                        egui::Shape::rect_filled(rect, 0, egui::Color32::BLACK),
                    )),
                    Item::Shape(egui::epaint::ClippedShape::new(rect, image_mesh)),
                ],
            )
            .unwrap();
            compositing::paint(&ui.ctx().debug_painter(), group).unwrap();
        },
    );
    (
        context.tessellate(output.shapes, output.pixels_per_point),
        output.textures_delta,
    )
}
fn upload(
    renderer: &mut egui_wgpu::Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut delta: egui::TexturesDelta,
) {
    for (id, deltas) in delta.set.drain() {
        for delta in deltas {
            renderer.update_texture(device, queue, id, &delta);
        }
    }
    for id in delta.free.drain() {
        renderer.free_texture(&id);
    }
    delta.clear();
}
#[test]
#[ignore = "requires an actual native wgpu adapter; run with --ignored --nocapture"]
fn current_frame_producers_cleanup_and_revocation() {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("native adapter");
    println!("GPU contract adapter: {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let context = egui::Context::default();
    let mut host = Host::new(&context, &device).unwrap();
    assert!(matches!(
        Host::new(&context, &device),
        Err(Error::AlreadyInstalled)
    ));
    let mut renderer = egui_wgpu::Renderer::new(
        &device,
        wgpu::TextureFormat::Rgba8Unorm,
        egui_wgpu::RendererOptions {
            dithering: false,
            ..Default::default()
        },
    );
    let source = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("caller owned source"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = source.create_view(&Default::default());
    let source_id = renderer.register_native_texture(&device, &view, wgpu::FilterMode::Nearest);
    for mode in 0..3 {
        let mut clear = device.create_command_encoder(&Default::default());
        {
            let _pass = clear.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        queue.submit([clear.finish()]);
        let (mut jobs, delta) = jobs(&context, source_id);
        upload(&mut renderer, &device, &queue, delta);
        // Put the producer after the group to ensure UI order isn't mistaken for GPU preparation order.
        let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(32.0, 32.0));
        jobs.push(egui::ClippedPrimitive {
            clip_rect: rect,
            primitive: Primitive::Callback(Callback::new_paint_callback(
                rect,
                Producer {
                    view: view.clone(),
                    mode,
                },
            )),
        });
        let mut frame = host.prepare_frame(&mut renderer, jobs, screen()).unwrap();
        let temporary = frame.registered.clone();
        assert_eq!(temporary.len(), 1);
        let mut capture = egui_wgpu::capture::CaptureState::new(
            &device,
            wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let target = capture.texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        let commands = frame.encode(&queue, &mut encoder).unwrap();
        assert!(frame.encode(&queue, &mut encoder).is_err());
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            frame.render(&mut pass).unwrap();
        }
        let buffer = capture.copy_textures(&device, None, &mut encoder);
        queue.submit(commands.into_iter().chain([encoder.finish()]));
        drop(frame);
        for id in temporary {
            assert!(renderer.texture(&id).is_none());
        }
        assert!(renderer.texture(&source_id).is_some());
        assert_eq!(host.pool.len(), 1);
        let (tx, rx) = std::sync::mpsc::channel();
        capture.read_screen_rgba(
            buffer,
            vec![egui::ScreenshotCallback::new(move |image| {
                tx.send(image).unwrap()
            })],
        );
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            })
            .unwrap();
        let image = rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
        assert_eq!(
            image[(16, 16)],
            egui::Color32::WHITE,
            "producer mode {mode}"
        );
        assert!((127..=128).contains(&image[(5, 5)].r()));
    }
    // A malformed binding fails and automatically releases temporary registrations.
    let (missing, delta) = jobs(&context, egui::TextureId::User(u64::MAX));
    upload(&mut renderer, &device, &queue, delta);
    assert!(
        host.prepare_frame(&mut renderer, missing, screen())
            .is_err()
    );
    let (old, delta) = jobs(&context, source_id);
    upload(&mut renderer, &device, &queue, delta);
    host.invalidate();
    assert_eq!(host.check(), Err(Error::ContextLost));
    let mut replacement = Host::new(&context, &device).unwrap();
    let frame = replacement
        .prepare_frame(&mut renderer, old, screen())
        .unwrap();
    assert!(frame.jobs.is_empty());
    drop(frame);
    assert!(replacement.pool.is_empty());
    assert!(renderer.texture(&source_id).is_some());
    // Deliberately dropped, unencoded frames still unregister their temporary images.
    let (cancelled, delta) = jobs(&context, source_id);
    upload(&mut renderer, &device, &queue, delta);
    let frame = replacement
        .prepare_frame(&mut renderer, cancelled, screen())
        .unwrap();
    let temporary = frame.registered.clone();
    drop(frame);
    for id in temporary {
        assert!(renderer.texture(&id).is_none());
    }
    renderer.free_texture(&source_id);
    println!(
        "GPU contracts passed: all producer paths, final pixels, cancellation, source lifetime, restart revocation"
    );
}
