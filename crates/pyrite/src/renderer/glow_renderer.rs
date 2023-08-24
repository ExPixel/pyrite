use std::{num::NonZeroU32, sync::Arc};

use anyhow::Context as _;
use gba::video::{VISIBLE_LINE_COUNT, VISIBLE_LINE_WIDTH};
use glow::{Buffer, HasContext, Program, Texture, VertexArray};
use glutin::{
    config::{Config as GlutinConfig, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
    },
    display::GetGlDisplay,
    prelude::{
        GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContextGlSurfaceAccessor,
    },
    surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
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

pub struct GlowApplication;

impl Application for GlowApplication {
    type Resources = Resources;

    fn init(context: AppInitContext) -> anyhow::Result<Self::Resources> {
        tracing::debug!("initializing glow renderer");
        let resources = init_window(context.config, context.event_loop)
            .context("error while initializing window and context")?;
        Ok(resources)
    }

    fn handle_event(context: AppEventContext<Self::Resources>) -> anyhow::Result<()> {
        let AppEventContext {
            event,
            resources,
            config,
            gba,
            event_loop_window_target,
            ..
        } = context;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width != 0 && size.height != 0 {
                    if let Some((context, surface)) = resources.context.context_and_surface() {
                        surface.resize(
                            context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }

                    if let Some((gl, window)) = resources.context.gl_and_window() {
                        let window_size = window.inner_size();
                        unsafe {
                            gl.viewport(0, 0, window_size.width as _, window_size.height as _)
                        };
                    }
                }

                if let Some(window) = resources.context.window() {
                    window.request_redraw();
                }
            }

            Event::RedrawRequested(_) => {
                resources
                    .context
                    .ensure_current()
                    .context("error while ensuring current")?;

                if let Some(gl) = resources.context.gl() {
                    render_gba(gba, gl, &mut resources.gba);
                }

                if let Some((context, surface)) = resources.context.context_and_surface() {
                    surface
                        .swap_buffers(context)
                        .context("error while swapping buffers")?;
                }
            }

            Event::Resumed => {
                match resources.context {
                    ContextType::NotCurrent { ref mut window, .. } if window.is_none() => {
                        let window_builder = new_window_builder(config);
                        *window = Some(Arc::new(
                            glutin_winit::finalize_window(
                                event_loop_window_target,
                                window_builder,
                                &resources.gl_config,
                            )
                            .context("error while finalizing window in resumed stage")?,
                        ));
                        tracing::debug!("finalized window");
                    }
                    _ => {}
                }

                if !resources.window_initialized {
                    let window = resources.context.window().unwrap();
                    let gba_window = window.clone();

                    let attrs = window.build_surface_attributes(Default::default());
                    let gl_surface = unsafe {
                        resources
                            .gl_config
                            .display()
                            .create_window_surface(&resources.gl_config, &attrs)
                            .unwrap()
                    };
                    resources
                        .context
                        .make_current(gl_surface)
                        .context("error while setting window context")?;

                    if let Some((context, surface)) = resources.context.context_and_surface() {
                        if let Err(err) = surface.set_swap_interval(
                            context,
                            SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
                        ) {
                            tracing::error!(error = debug(err), "error while setting vsync");
                        }
                    }

                    gba.write().request_repaint =
                        Some(Box::new(move |_, _| gba_window.request_redraw()));

                    resources.window_initialized = true;

                    if let Some((gl, window)) = resources.context.gl_and_window() {
                        let window_size = window.inner_size();
                        unsafe {
                            gl.viewport(0, 0, window_size.width as _, window_size.height as _)
                        };
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }
}

fn render_gba(gba: &SharedGba, gl: &glow::Context, resources: &mut Option<GbaResources>) {
    let resources = resources.get_or_insert_with(|| unsafe {
        let vertex_shader = gl
            .create_shader(glow::VERTEX_SHADER)
            .map_err(anyhow::Error::msg)
            .expect("error while creating vertex shader");
        gl.shader_source(vertex_shader, GL_VERT_SHADER_SRC);
        gl.compile_shader(vertex_shader);
        if !gl.get_shader_compile_status(vertex_shader) {
            let shader_error = gl.get_shader_info_log(vertex_shader);
            panic!("vertex shader error: {shader_error}");
        }

        let fragment_shader = gl
            .create_shader(glow::FRAGMENT_SHADER)
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
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice::<_, u8>(&GL_DEFAULT_VERTICES),
            glow::STATIC_DRAW,
        );
        tracing::debug!("GBA screen vertex buffer created");

        let vertex_array = gl
            .create_vertex_array()
            .map_err(anyhow::Error::msg)
            .expect("error while creating vertex array");
        gl.bind_vertex_array(Some(vertex_array));
        let sz_float = std::mem::size_of::<f32>() as i32;
        let pos = gl
            .get_attrib_location(program, "in_position")
            .expect("no in_position attribute");
        let tex = gl
            .get_attrib_location(program, "in_texcoord")
            .expect("no in_texcoord attribute");
        gl.vertex_attrib_pointer_f32(pos, 2, glow::FLOAT, false, 4 * sz_float, 0);
        gl.vertex_attrib_pointer_f32(tex, 2, glow::FLOAT, false, 4 * sz_float, 2 * sz_float);
        gl.enable_vertex_attrib_array(pos);
        gl.enable_vertex_attrib_array(tex);
        tracing::debug!("GBA screen vertex array object created");

        let texture = gl
            .create_texture()
            .map_err(anyhow::Error::msg)
            .expect("error while creating texture");

        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        let mut gba_data = gba.write();
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as _,
            VISIBLE_LINE_WIDTH as _,
            VISIBLE_LINE_COUNT as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_SHORT_1_5_5_5_REV,
            Some(bytemuck::cast_slice(&gba_data.ready_buffer[..])),
        );
        gba_data.painted = true;
        drop(gba_data);

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as _,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as _,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as _,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as _,
        );

        GbaResources {
            texture,
            program,
            buffer,
            vertex_array,
        }
    });

    unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(resources.buffer));
        gl.bind_vertex_array(Some(resources.vertex_array));
        gl.use_program(Some(resources.program));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(resources.texture));
    }

    gba.with_mut(|g| {
        if !g.painted {
            let buffer = if g.current_mode == GbaRunMode::Step
                && g.gba.mapped.video.current_scanline() < 160
            {
                bytemuck::cast_slice(&g.frame_buffer[..])
            } else {
                bytemuck::cast_slice(&g.ready_buffer[..])
            };

            unsafe {
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    VISIBLE_LINE_WIDTH as _,
                    VISIBLE_LINE_COUNT as _,
                    glow::RGBA,
                    glow::UNSIGNED_SHORT_1_5_5_5_REV,
                    glow::PixelUnpackData::Slice(buffer),
                );
            }
        }
        g.painted = true;
    });

    unsafe { gl.draw_arrays(glow::TRIANGLES, 0, 6) };
}

fn new_window_builder(config: &SharedConfig) -> WindowBuilder {
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
}

fn init_window(config: &SharedConfig, event_loop: &EventLoop<()>) -> anyhow::Result<Resources> {
    let window_builder = new_window_builder(config);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let (window, gl_config) = display_builder
        .build(event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .map_err(|err| anyhow::anyhow!("glutin error: {err:?}"))
        .context("error while building window")?;
    tracing::debug!("picked a config with {} samples", gl_config.num_samples());

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());
    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 2))))
        .build(raw_window_handle);
    let gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .and_then(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .and_then(|_| gl_display.create_context(&gl_config, &legacy_context_attributes))
            })
    }
    .context("error while creating GL context")?;

    Ok(Resources {
        context: ContextType::NotCurrent {
            window: window.map(Arc::new),
            context: gl_context,
            gl: None,
        },
        gba: None,
        gl_config,
        window_initialized: false,
    })
}

pub struct Resources {
    context: ContextType,
    gba: Option<GbaResources>,
    gl_config: GlutinConfig,
    window_initialized: bool,
}

struct GbaResources {
    texture: Texture,
    program: Program,
    buffer: Buffer,
    vertex_array: VertexArray,
}

impl GbaResources {
    fn destroy(self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vertex_array);
            gl.delete_buffer(self.buffer);
            gl.delete_texture(self.texture);
            gl.delete_program(self.program);
        }
        tracing::debug!("destroyed GBA screen resources");
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        if let Some(gba) = self.gba.take() {
            if self.context.is_possibly_current() {
                self.context
                    .ensure_current()
                    .expect("failed to ensure current");
                gba.destroy(self.context.gl().expect("no GL context"));
            }
        }
    }
}

impl ResourcesCommon for Resources {
    fn window(&self) -> Option<&Window> {
        self.context.window().map(|w| &**w)
    }
}

enum ContextType {
    NotCurrent {
        context: NotCurrentContext,
        window: Option<Arc<Window>>,
        gl: Option<glow::Context>,
    },

    PossiblyCurrent {
        context: PossiblyCurrentContext,
        window: Arc<Window>,
        surface: Surface<WindowSurface>,
        gl: glow::Context,
    },

    None,
}

impl ContextType {
    pub fn is_possibly_current(&self) -> bool {
        matches!(self, ContextType::PossiblyCurrent { .. })
    }

    pub fn ensure_current(&mut self) -> anyhow::Result<()> {
        match self {
            ContextType::NotCurrent { .. } => {
                anyhow::bail!("ensure current must be called with possibly current context")
            }

            ContextType::PossiblyCurrent {
                context, surface, ..
            } => context
                .make_current(surface)
                .context("error while ensuring context is current"),

            ContextType::None => {
                anyhow::bail!("ensure current must be called with possibly current context")
            }
        }
    }

    pub fn make_current(&mut self, surface: Surface<WindowSurface>) -> anyhow::Result<()> {
        let tmp = std::mem::replace(self, ContextType::None);
        *self = match tmp {
            ContextType::NotCurrent {
                context,
                window: Some(window),
                gl,
            } => {
                let context = context
                    .make_current(&surface)
                    .context("error while making not current window current")?;
                let display = context.display();
                let gl = gl.unwrap_or_else(|| unsafe {
                    glow::Context::from_loader_function_cstr(|s| {
                        display.get_proc_address(s) as *const _
                    })
                });
                ContextType::PossiblyCurrent {
                    context,
                    window,
                    surface,
                    gl,
                }
            }

            ContextType::PossiblyCurrent {
                context,
                window,
                surface,
                gl,
            } => {
                context
                    .make_current(&surface)
                    .context("error while making not current window current")?;
                ContextType::PossiblyCurrent {
                    context,
                    window,
                    surface,
                    gl,
                }
            }

            ContextType::NotCurrent { .. } => {
                anyhow::bail!("bad state; called make current with no window")
            }
            ContextType::None => anyhow::bail!("bad state; context type = none"),
        };
        Ok(())
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        match self {
            ContextType::NotCurrent { ref window, .. } => window.as_ref(),
            ContextType::PossiblyCurrent { ref window, .. } => Some(window),
            ContextType::None => unimplemented!("bad state; context type = none"),
        }
    }

    pub fn context_and_surface(
        &self,
    ) -> Option<(&PossiblyCurrentContext, &Surface<WindowSurface>)> {
        match self {
            ContextType::NotCurrent { .. } => None,
            ContextType::PossiblyCurrent {
                ref context,
                ref surface,
                ..
            } => Some((context, surface)),
            ContextType::None => unimplemented!("bad state; context type = none"),
        }
    }

    pub fn gl_and_window(&self) -> Option<(&glow::Context, &Window)> {
        match self {
            ContextType::NotCurrent { .. } => None,
            ContextType::PossiblyCurrent {
                ref gl, ref window, ..
            } => Some((gl, window)),
            ContextType::None => unimplemented!("bad state; context type = none"),
        }
    }

    #[allow(dead_code)]
    pub fn surface(&self) -> Option<&Surface<WindowSurface>> {
        match self {
            ContextType::NotCurrent { .. } => None,
            ContextType::PossiblyCurrent { ref surface, .. } => Some(surface),
            ContextType::None => unimplemented!("bad state; context type = none"),
        }
    }

    pub fn gl(&self) -> Option<&glow::Context> {
        match self {
            ContextType::NotCurrent { .. } => None,
            ContextType::PossiblyCurrent { ref gl, .. } => Some(gl),
            ContextType::None => unimplemented!("bad state; context type = none"),
        }
    }
}

#[rustfmt::skip]
const GL_DEFAULT_VERTICES: [f32; 24] = [
    -1.0,  1.0, 0.0, 0.0, // left, top
     1.0,  1.0, 1.0, 0.0, // right, top
    -1.0, -1.0, 0.0, 1.0, // left, bottom
    -1.0, -1.0, 0.0, 1.0, // left, bottom
     1.0, -1.0, 1.0, 1.0, // right, bottom
     1.0,  1.0, 1.0, 0.0, // right, top
];

const GL_FRAG_SHADER_SRC: &str = "\
#version 140
in vec2 frag_texcoord;
out vec4 out_color;
uniform sampler2D tex;
void main() {
    vec4 col = texture(tex, frag_texcoord);
    out_color = vec4(col.rgb, 1.0);
}";

const GL_VERT_SHADER_SRC: &str = "\
#version 140
in vec2 in_position;
in vec2 in_texcoord;
out vec2 frag_texcoord;
void main() {
    gl_Position = vec4(in_position, 0.0, 1.0);
    frag_texcoord = in_texcoord;
}";
