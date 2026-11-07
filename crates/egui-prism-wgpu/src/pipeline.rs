use crate::failure;
use egui_prism::{
    compositing::Error,
    egui::{
        Rect,
        epaint::{Mesh, Vertex},
    },
};
use egui_wgpu::wgpu::{self, util::DeviceExt as _};
pub(super) struct Target {
    pub size: [u32; 2],
    pub view: wgpu::TextureView,
}
impl Target {
    pub fn new(device: &wgpu::Device, size: [u32; 2]) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("prism isolated group"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Self {
            size,
            view: texture.create_view(&Default::default()),
        }
    }
}
pub(super) struct Draw {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
    locals: wgpu::BindGroup,
    image: wgpu::BindGroup,
    scissor: [u32; 4],
}
pub(super) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    locals: wgpu::BindGroupLayout,
}
impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let locals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prism locals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(32),
                },
                count: None,
            }],
        });
        // Structural compatibility with egui-wgpu 0.37's public texture binding.
        // Original texture/sampler bindings preserve hardware filtering and wrap modes.
        let images = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prism egui images"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(16),
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prism layout"),
            bind_group_layouts: &[Some(&locals), Some(&images)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("prism shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("group.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("prism pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Unorm8x4],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline, locals }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn mesh(
        &self,
        device: &wgpu::Device,
        mesh: &Mesh,
        image: wgpu::BindGroup,
        rect: Rect,
        clip: Rect,
        ppp: f32,
        opacity: f32,
    ) -> Result<Option<Draw>, Error> {
        if mesh.indices.is_empty() || !clip.is_positive() {
            return Ok(None);
        }
        if !mesh.is_valid()
            || mesh
                .vertices
                .iter()
                .any(|v| !v.pos.is_finite() || !v.uv.is_finite())
        {
            return Err(failure("invalid tessellated mesh"));
        }
        let min = ((clip.min - rect.min) * ppp)
            .round()
            .max(egui_prism::egui::Vec2::ZERO);
        let max = ((clip.max - rect.min) * ppp).round().min(rect.size() * ppp);
        if max.x <= min.x || max.y <= min.y {
            return Ok(None);
        }
        let uniform = [
            rect.min.x,
            rect.min.y,
            rect.width(),
            rect.height(),
            opacity,
            0.0,
            0.0,
            0.0,
        ];
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("prism transform"),
            contents: bytemuck::cast_slice(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let locals = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prism transform"),
            layout: &self.locals,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(Some(Draw {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("prism vertices"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("prism indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            count: u32::try_from(mesh.indices.len()).map_err(|_| failure("mesh too large"))?,
            locals,
            image,
            scissor: [
                min.x as u32,
                min.y as u32,
                (max.x - min.x) as u32,
                (max.y - min.y) as u32,
            ],
        }))
    }
    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, target: &Target, draws: &[Draw]) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("prism isolated pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        for draw in draws {
            pass.set_bind_group(0, &draw.locals, &[]);
            pass.set_bind_group(1, &draw.image, &[]);
            pass.set_vertex_buffer(0, draw.vertices.slice(..));
            pass.set_index_buffer(draw.indices.slice(..), wgpu::IndexFormat::Uint32);
            let [x, y, w, h] = draw.scissor;
            pass.set_scissor_rect(x, y, w, h);
            pass.draw_indexed(0..draw.count, 0, 0..1);
        }
    }
}
