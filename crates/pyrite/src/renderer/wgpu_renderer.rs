use std::sync::Arc;

use anyhow::Context as _;
use gba::video::{VISIBLE_LINE_COUNT, VISIBLE_LINE_WIDTH};
use wgpu::{
    util::DeviceExt as _, BindGroup, Buffer, Device, RenderPass, RenderPipeline, Surface,
    SurfaceConfiguration, Texture,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{
    config::SharedConfig,
    gba_runner::{GbaRunMode, SharedGba},
};

use super::common::{AppEventContext, AppInitContext, Application, ResourcesCommon};

pub struct WgpuApplication;

impl Application for WgpuApplication {
    type Resources = Resources;

    fn init(context: AppInitContext) -> anyhow::Result<Self::Resources> {
        pollster::block_on(async move {
            init_resources(context.config, context.gba.clone(), context.event_loop)
                .await
                .context("error while initializing WGPU resources")
        })
    }

    fn handle_event(context: AppEventContext<Self::Resources>) -> anyhow::Result<()> {
        let AppEventContext {
            event,
            resources,
            gba,
            control_flow,
            ..
        } = context;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                resources.surface_config.width = size.width;
                resources.surface_config.height = size.height;
                resources
                    .surface
                    .configure(&resources.device, &resources.surface_config);
                resources.window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                let frame = resources
                    .surface
                    .get_current_texture()
                    .context("error while acquiring next swapchain frame texture")?;
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = resources
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    render_gba(gba, &mut render_pass, resources);
                }

                resources.queue.submit(Some(encoder.finish()));
                frame.present();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }

            _ => {}
        }

        Ok(())
    }
}

fn render_gba<'r>(gba: &SharedGba, render_pass: &mut RenderPass<'r>, resources: &'r Resources) {
    let gba_resources = &resources.gba;

    gba.with_mut(|g| {
        if !g.painted {
            let texture_size = wgpu::Extent3d {
                width: VISIBLE_LINE_WIDTH as u32,
                height: VISIBLE_LINE_COUNT as u32,
                depth_or_array_layers: 1,
            };

            let buffer = if g.current_mode == GbaRunMode::Step
                && g.gba.mapped.video.current_scanline() < 160
            {
                bytemuck::cast_slice(&g.frame_buffer[..])
            } else {
                bytemuck::cast_slice(&g.ready_buffer[..])
            };

            resources.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &resources.gba.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                buffer,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(2 * texture_size.width),
                    rows_per_image: Some(texture_size.height),
                },
                texture_size,
            );

            g.painted = true;
        }
    });

    render_pass.set_pipeline(&gba_resources.render_pipeline);
    render_pass.set_vertex_buffer(0, gba_resources.vertex_buffer.slice(..));
    render_pass.set_bind_group(0, &gba_resources.bind_group, &[]);
    render_pass.draw(0..6, 0..1);
}

async fn init_resources(
    config: &SharedConfig,
    gba: SharedGba,
    event_loop: &EventLoop<()>,
) -> anyhow::Result<Resources> {
    let window = {
        let config = config.read();
        WindowBuilder::new()
            .with_title("Pyrite")
            .with_inner_size(LogicalSize::new(
                config.gui.window_width.unwrap_or(VISIBLE_LINE_WIDTH as u32),
                config
                    .gui
                    .window_height
                    .unwrap_or(VISIBLE_LINE_COUNT as u32),
            ))
            .build(event_loop)
            .context("error occurred while building window")?
    };

    let instance = wgpu::Instance::default();
    let surface =
        unsafe { instance.create_surface(&window) }.context("error while creating wgpu surface")?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .context("error finding appropriate adapter")?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .context("error while creating device")?;

    let texture_size = wgpu::Extent3d {
        width: VISIBLE_LINE_WIDTH as u32,
        height: VISIBLE_LINE_COUNT as u32,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R16Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("gba_screen_texture"),
        view_formats: &[],
    });

    let mut gba_data = gba.write();
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&gba_data.ready_buffer[..]),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(2 * texture_size.width),
            rows_per_image: Some(texture_size.height),
        },
        texture_size,
    );
    gba_data.painted = true;
    drop(gba_data);

    tracing::debug!("GBA screen wgpu texture created");

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("gba_screen_texture_view"),
        ..Default::default()
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        label: Some("gba_screen_texture_sampler"),
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("gba_screen_texture_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("gba_screen_texture_bind_group"),
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(WGPU_DEFAULT_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gba_screen_texture_shader"),
        source: wgpu::ShaderSource::Wgsl(WGPU_SHADER_SRC.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities
        .formats
        .iter()
        .find(|f| {
            matches!(
                f,
                wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
            )
        })
        .ok_or_else(|| anyhow::anyhow!("no suitable surface format"))?;
    let swapchain_format = *swapchain_format;
    tracing::debug!("using swapchain format {swapchain_format:?}");

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let window_size = window.inner_size();
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &surface_config);

    let gba_resources = GbaResources {
        texture,
        bind_group,
        render_pipeline,
        vertex_buffer,
    };

    let resources = Resources {
        window: Arc::new(window),
        device,
        surface,
        surface_config,
        queue,
        gba: gba_resources,
    };

    let gba_window = resources.window.clone();
    gba.write().request_repaint = Some(Box::new(move |_, _| gba_window.request_redraw()));

    Ok(resources)
}

pub struct Resources {
    window: Arc<Window>,

    device: Device,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    queue: wgpu::Queue,

    gba: GbaResources,
}

struct GbaResources {
    texture: Texture,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl ResourcesCommon for Resources {
    fn window(&self) -> Option<&Window> {
        Some(&*self.window)
    }
}

#[rustfmt::skip]
const WGPU_DEFAULT_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // left, bottom
    Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // right, top
    Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0] }, // left, top
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // left, bottom
    Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] }, // right, bottom
    Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // right, top
];

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
    let r: f32 = f32( c             & u32(31)) / f32(31.0);
    let g: f32 = f32((c >> u32( 5)) & u32(31)) / f32(31.0);
    let b: f32 = f32((c >> u32(10)) & u32(31)) / f32(31.0);
    return vec4(r, g, b, 1.0);
}";

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
