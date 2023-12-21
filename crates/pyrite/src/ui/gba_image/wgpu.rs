use eframe::{
    egui_wgpu::{Callback, CallbackTrait},
    wgpu::{
        util::DeviceExt, BindGroup, Buffer, Extent3d, RenderPipeline, Texture, TextureDescriptor,
        TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    },
};
use egui::PaintCallback;
use gba::video::{VISIBLE_LINE_COUNT, VISIBLE_LINE_WIDTH};

use crate::gba_runner::{GbaRunMode, SharedGba};

pub struct GbaImageWgpu {
    callback: PaintCallback,
}

impl GbaImageWgpu {
    pub fn new(gba: SharedGba) -> anyhow::Result<Self> {
        let wgpu_painter = WgpuPainter::new(gba);
        let callback = Callback::new_paint_callback(egui::Rect::NOTHING, wgpu_painter);
        Ok(Self { callback })
    }

    pub fn paint(&mut self, rect: egui::Rect) -> egui::PaintCallback {
        let mut callback = self.callback.clone();
        callback.rect = rect;
        callback
    }

    pub fn destroy(&mut self) {
        /* NOP */
    }
}

struct WgpuPainterResources {
    texture: Texture,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

struct WgpuPainter {
    gba: SharedGba,
}

impl WgpuPainter {
    fn new(gba: SharedGba) -> Self {
        Self { gba }
    }
}

impl CallbackTrait for WgpuPainter {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        if callback_resources.contains::<WgpuPainterResources>() {
            return Vec::new();
        }

        let texture_size = Extent3d {
            width: VISIBLE_LINE_WIDTH as u32,
            height: VISIBLE_LINE_COUNT as u32,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R16Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("gba_screen_texture"),
            view_formats: &[],
        });

        let mut gba_data = self.gba.write();
        queue.write_texture(
            eframe::wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: eframe::wgpu::Origin3d::ZERO,
                aspect: eframe::wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&gba_data.ready_buffer[..]),
            eframe::wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(2 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );
        gba_data.painted = true;
        drop(gba_data);
        tracing::debug!("GBA screen wgpu texture initialized");

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("gba_screen_texture_view"),
            ..Default::default()
        });

        let sampler = device.create_sampler(&eframe::wgpu::SamplerDescriptor {
            address_mode_u: eframe::wgpu::AddressMode::ClampToEdge,
            address_mode_v: eframe::wgpu::AddressMode::ClampToEdge,
            address_mode_w: eframe::wgpu::AddressMode::ClampToEdge,
            mag_filter: eframe::wgpu::FilterMode::Nearest,
            min_filter: eframe::wgpu::FilterMode::Nearest,
            mipmap_filter: eframe::wgpu::FilterMode::Nearest,
            label: Some("gba_screen_texture_sampler"),
            ..Default::default()
        });

        let bind_group_layout =
            device.create_bind_group_layout(&eframe::wgpu::BindGroupLayoutDescriptor {
                label: Some("gba_screen_texture_bind_group_layout"),
                entries: &[
                    eframe::wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: eframe::wgpu::ShaderStages::FRAGMENT,
                        ty: eframe::wgpu::BindingType::Texture {
                            sample_type: eframe::wgpu::TextureSampleType::Uint,
                            view_dimension: eframe::wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    eframe::wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: eframe::wgpu::ShaderStages::FRAGMENT,
                        ty: eframe::wgpu::BindingType::Sampler(
                            eframe::wgpu::SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&eframe::wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                eframe::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: eframe::wgpu::BindingResource::TextureView(&texture_view),
                },
                eframe::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: eframe::wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("gba_screen_texture_bind_group"),
        });

        let vertex_buffer = device.create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(WGPU_DEFAULT_VERTICES),
            usage: eframe::wgpu::BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("gba_screen_texture_shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(WGPU_SHADER_SRC.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&eframe::wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: eframe::wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(eframe::wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(eframe::wgpu::ColorTargetState {
                        format: eframe::wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(eframe::wgpu::BlendState::REPLACE),
                        write_mask: eframe::wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: eframe::wgpu::PrimitiveState {
                    topology: eframe::wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: eframe::wgpu::FrontFace::Ccw,
                    cull_mode: Some(eframe::wgpu::Face::Back),
                    polygon_mode: eframe::wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: eframe::wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        callback_resources.insert(WgpuPainterResources {
            texture,
            bind_group,
            render_pipeline,
            vertex_buffer,
        });
        tracing::debug!("GBA screen wgpu resources initialized");

        Vec::new()
    }

    fn finish_prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let Some(resources) = callback_resources.get::<WgpuPainterResources>() else {
            return Vec::new();
        };

        let mut gba_data = self.gba.write();
        if !gba_data.painted {
            let texture_size = eframe::wgpu::Extent3d {
                width: VISIBLE_LINE_WIDTH as u32,
                height: VISIBLE_LINE_COUNT as u32,
                depth_or_array_layers: 1,
            };

            let buffer = if gba_data.current_mode == GbaRunMode::Frame {
                bytemuck::cast_slice(&gba_data.ready_buffer[..])
            } else {
                bytemuck::cast_slice(&gba_data.frame_buffer[..])
            };

            queue.write_texture(
                eframe::wgpu::ImageCopyTexture {
                    texture: &resources.texture,
                    mip_level: 0,
                    origin: eframe::wgpu::Origin3d::ZERO,
                    aspect: eframe::wgpu::TextureAspect::All,
                },
                buffer,
                eframe::wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(2 * texture_size.width),
                    rows_per_image: Some(texture_size.height),
                },
                texture_size,
            );
            gba_data.painted = true;
        }
        drop(gba_data);
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<WgpuPainterResources>() else {
            return;
        };

        render_pass.set_pipeline(&resources.render_pipeline);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &resources.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

#[cfg(feature = "wgpu")]
#[rustfmt::skip]
const WGPU_DEFAULT_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // left, bottom
    Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // right, top
    Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0] }, // left, top
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // left, bottom
    Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] }, // right, bottom
    Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // right, top
];

#[cfg(feature = "wgpu")]
const WGPU_SHADER_SRC: &str = "\
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<u32>;

@group(0) @binding(1)
var sam: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c: u32 = textureLoad(tex, vec2(u32(240.0 * in.tex_coords.x), u32(160.0 * in.tex_coords.y)), 0).r;
    let r: f32 = f32( c        & u32(31)) / f32(31.0);
    let g: f32 = f32((c >> u32( 5)) & u32(31)) / f32(31.0);
    let b: f32 = f32((c >> u32(10)) & u32(31)) / f32(31.0);
    return vec4(r, g, b, 1.0);
}";

#[cfg(feature = "wgpu")]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

#[cfg(feature = "wgpu")]
impl Vertex {
    fn desc() -> eframe::wgpu::VertexBufferLayout<'static> {
        use std::mem;
        eframe::wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as eframe::wgpu::BufferAddress,
            step_mode: eframe::wgpu::VertexStepMode::Vertex,
            attributes: &[
                eframe::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: eframe::wgpu::VertexFormat::Float32x3,
                },
                eframe::wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as eframe::wgpu::BufferAddress,
                    shader_location: 1,
                    format: eframe::wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
