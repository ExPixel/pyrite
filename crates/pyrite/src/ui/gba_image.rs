use crate::gba_runner::SharedGba;
use std::{any::Any, sync::Arc};

#[cfg(feature = "wgpu")]
use eframe::wgpu::{
    BindGroup as WgBindGroup, Buffer as WgBuffer, RenderPipeline as WgRenderPipeline,
    Texture as WgTexture,
};

#[cfg(feature = "glow")]
use eframe::glow::{
    Buffer as GlBuffer, Context as GlContext, HasContext as _, Program as GlProgram,
    Texture as GlTexture, VertexArray as GlVertexArray,
};

pub struct GbaImage {
    callback: Arc<dyn Any + Send + Sync>,
}

impl GbaImage {
    #[cfg(feature = "glow")]
    pub fn new_glow(gba: SharedGba) -> anyhow::Result<Self> {
        use egui::mutex::Mutex;

        let image_context = Mutex::new(GbaImageGlow {
            gpu_data: None,
            gl: None,
        });

        let callback = eframe::egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl();
            let info: &mut GbaImageGlow = &mut image_context.lock();
            let (ref texture, ref program, ref buffer, ref array) =
                info.gpu_data.get_or_insert_with(|| unsafe {
                    let vertex_shader = gl
                        .create_shader(eframe::glow::VERTEX_SHADER)
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating vertex shader");
                    gl.shader_source(vertex_shader, GL_VERT_SHADER_SRC);
                    gl.compile_shader(vertex_shader);
                    if !gl.get_shader_compile_status(vertex_shader) {
                        let shader_error = gl.get_shader_info_log(vertex_shader);
                        panic!("vertex shader error: {shader_error}");
                    }

                    let fragment_shader = gl
                        .create_shader(eframe::glow::FRAGMENT_SHADER)
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating vertex shader");
                    gl.shader_source(fragment_shader, GL_FRAG_SHADER_SRC);
                    gl.compile_shader(fragment_shader);
                    if !gl.get_shader_compile_status(fragment_shader) {
                        let shader_error = gl.get_shader_info_log(fragment_shader);
                        panic!("fragment shader error: {shader_error}");
                    }
                    tracing::debug!("GBA screen gl shaders compiled");

                    let program = gl
                        .create_program()
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating shader program");
                    gl.attach_shader(program, vertex_shader);
                    gl.attach_shader(program, fragment_shader);
                    gl.link_program(program);
                    if !gl.get_program_link_status(program) {
                        let program_error = gl.get_program_info_log(program);
                        panic!("program link error: {program_error}");
                    }
                    tracing::debug!("GBA screen gl shader program linked");

                    let buffer = gl
                        .create_buffer()
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating array buffer");
                    gl.bind_buffer(eframe::glow::ARRAY_BUFFER, Some(buffer));
                    gl.buffer_data_u8_slice(
                        eframe::glow::ARRAY_BUFFER,
                        bytemuck::cast_slice::<_, u8>(&GL_DEFAULT_VERTICES),
                        eframe::glow::STATIC_DRAW,
                    );
                    tracing::debug!("GBA screen vertex buffer created");

                    let array = gl
                        .create_vertex_array()
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating vertex array");
                    gl.bind_vertex_array(Some(array));
                    let sz_float = std::mem::size_of::<f32>() as i32;
                    let pos = gl
                        .get_attrib_location(program, "in_position")
                        .expect("no in_position attribute");
                    let tex = gl
                        .get_attrib_location(program, "in_texcoord")
                        .expect("no in_texcoord attribute");
                    gl.vertex_attrib_pointer_f32(
                        pos,
                        2,
                        eframe::glow::FLOAT,
                        false,
                        4 * sz_float,
                        0,
                    );
                    gl.vertex_attrib_pointer_f32(
                        tex,
                        2,
                        eframe::glow::FLOAT,
                        false,
                        4 * sz_float,
                        2 * sz_float,
                    );
                    gl.enable_vertex_attrib_array(pos);
                    gl.enable_vertex_attrib_array(tex);
                    tracing::debug!("GBA screen vertex array object created");

                    let texture = gl
                        .create_texture()
                        .map_err(anyhow::Error::msg)
                        .expect("error while creating texture");

                    gl.bind_texture(eframe::glow::TEXTURE_2D, Some(texture));

                    let mut gba_data = gba.write();
                    gl.tex_image_2d(
                        eframe::glow::TEXTURE_2D,
                        0,
                        eframe::glow::RGB as _,
                        240,
                        160,
                        0,
                        eframe::glow::RGBA,
                        eframe::glow::UNSIGNED_SHORT_1_5_5_5_REV,
                        Some(bytemuck::cast_slice(&gba_data.ready_buffer[..])),
                    );
                    gba_data.painted = true;
                    drop(gba_data);

                    gl.tex_parameter_i32(
                        eframe::glow::TEXTURE_2D,
                        eframe::glow::TEXTURE_WRAP_S,
                        eframe::glow::CLAMP_TO_EDGE as _,
                    );
                    gl.tex_parameter_i32(
                        eframe::glow::TEXTURE_2D,
                        eframe::glow::TEXTURE_WRAP_T,
                        eframe::glow::CLAMP_TO_EDGE as _,
                    );
                    gl.tex_parameter_i32(
                        eframe::glow::TEXTURE_2D,
                        eframe::glow::TEXTURE_MIN_FILTER,
                        eframe::glow::NEAREST as _,
                    );
                    gl.tex_parameter_i32(
                        eframe::glow::TEXTURE_2D,
                        eframe::glow::TEXTURE_MAG_FILTER,
                        eframe::glow::NEAREST as _,
                    );

                    info.gl = Some(gl.clone());
                    (texture, program, buffer, array)
                });

            unsafe {
                gl.bind_buffer(eframe::glow::ARRAY_BUFFER, Some(*buffer));
                gl.bind_vertex_array(Some(*array));
                gl.use_program(Some(*program));
                gl.active_texture(eframe::glow::TEXTURE0);
                gl.bind_texture(eframe::glow::TEXTURE_2D, Some(*texture));
            }

            let mut gba_data = gba.write();
            if !gba_data.painted {
                unsafe {
                    gl.tex_sub_image_2d(
                        eframe::glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        240,
                        160,
                        eframe::glow::RGBA,
                        eframe::glow::UNSIGNED_SHORT_1_5_5_5_REV,
                        eframe::glow::PixelUnpackData::Slice(bytemuck::cast_slice(
                            &gba_data.ready_buffer[..],
                        )),
                    );
                }
            }
            gba_data.painted = true;
            drop(gba_data);

            unsafe { gl.draw_arrays(eframe::glow::TRIANGLES, 0, 6) };
        });
        Ok(Self {
            callback: Arc::new(callback),
        })
    }

    #[cfg(feature = "wgpu")]
    pub fn new_wgpu(gba: SharedGba) -> anyhow::Result<Self> {
        // let gpu_data: Arc<egui::mutex::Mutex<Option<(WgTexture,)>>> = Arc::default();

        use eframe::wgpu::util::DeviceExt;

        use crate::gba_runner::GbaRunMode;

        let callback = eframe::egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _command_encoder, type_map| {
                let gpu_data = if let Some(gpu_data) = type_map.get_mut::<GbaImageWgpuData>() {
                    gpu_data
                } else {
                    let texture_size = eframe::wgpu::Extent3d {
                        width: VISIBLE_LINE_WIDTH as u32,
                        height: VISIBLE_LINE_COUNT as u32,
                        depth_or_array_layers: 1,
                    };

                    let texture = device.create_texture(&eframe::wgpu::TextureDescriptor {
                        size: texture_size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: eframe::wgpu::TextureDimension::D2,
                        format: eframe::wgpu::TextureFormat::R16Uint,
                        usage: eframe::wgpu::TextureUsages::TEXTURE_BINDING
                            | eframe::wgpu::TextureUsages::COPY_DST,
                        label: Some("gba_screen_texture"),
                        view_formats: &[],
                    });

                    let mut gba_data = gba.write();
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

                    tracing::debug!("GBA screen wgpu texture created");

                    let texture_view = texture.create_view(&eframe::wgpu::TextureViewDescriptor {
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

                    let vertex_buffer =
                        device.create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(WGPU_DEFAULT_VERTICES),
                            usage: eframe::wgpu::BufferUsages::VERTEX,
                        });

                    let shader =
                        device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
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

                    let gpu_data = GbaImageWgpuData {
                        texture,
                        bind_group,
                        render_pipeline,
                        vertex_buffer,
                    };
                    type_map.insert(gpu_data);
                    type_map.get_mut::<GbaImageWgpuData>().unwrap()
                };

                let mut gba_data = gba.write();
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
                            texture: &gpu_data.texture,
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
            })
            .paint(move |_info, render_pass, type_map| {
                let Some(gpu_data) = type_map.get::<GbaImageWgpuData>() else {
                    tracing::error!("gpu image data not found in wgpu renderer type map");
                    return;
                };
                render_pass.set_pipeline(&gpu_data.render_pipeline);
                render_pass.set_vertex_buffer(0, gpu_data.vertex_buffer.slice(..));
                render_pass.set_bind_group(0, &gpu_data.bind_group, &[]);
                render_pass.draw(0..6, 0..1);
            });

        Ok(Self {
            callback: Arc::new(callback),
        })
    }

    pub(crate) fn callback(&self) -> Arc<dyn Any + Send + Sync> {
        self.callback.clone()
    }
}

#[cfg(feature = "glow")]
struct GbaImageGlow {
    gpu_data: Option<(GlTexture, GlProgram, GlBuffer, GlVertexArray)>,
    gl: Option<Arc<GlContext>>,
}

#[cfg(feature = "glow")]
impl Drop for GbaImageGlow {
    fn drop(&mut self) {
        if let Some((texture, program, buffer, array)) = self.gpu_data.take() {
            if let Some(gl) = self.gl.take() {
                unsafe {
                    gl.delete_vertex_array(array);
                    gl.delete_buffer(buffer);
                    gl.delete_texture(texture);
                    gl.delete_program(program);
                }
                tracing::debug!("destroyed GBA screen GL resources");
            }
        }
    }
}

#[cfg(feature = "wgpu")]
pub struct GbaImageWgpuData {
    texture: WgTexture,
    bind_group: WgBindGroup,
    render_pipeline: WgRenderPipeline,
    vertex_buffer: WgBuffer,
}

#[cfg(feature = "glow")]
const GL_FRAG_SHADER_SRC: &str = "\
#version 150 core
in vec2 frag_texcoord;
out vec4 out_color;
uniform sampler2D tex;
void main() {
    vec4 col = texture(tex, frag_texcoord);
    out_color = vec4(col.rgb, 1.0);
}";

#[cfg(feature = "glow")]
const GL_VERT_SHADER_SRC: &str = "\
#version 150 core
in vec2 in_position;
in vec2 in_texcoord;
out vec2 frag_texcoord;
void main() {
    gl_Position = vec4(in_position, 0.0, 1.0);
    frag_texcoord = in_texcoord;
}";

#[cfg(feature = "glow")]
#[rustfmt::skip]
const GL_DEFAULT_VERTICES: [f32; 24] = [
    -1.0,  1.0, 0.0, 0.0, // left, top
     1.0,  1.0, 1.0, 0.0, // right, top
    -1.0, -1.0, 0.0, 1.0, // left, bottom
    -1.0, -1.0, 0.0, 1.0, // left, bottom
     1.0, -1.0, 1.0, 1.0, // right, bottom
     1.0,  1.0, 1.0, 0.0, // right, top
];

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
