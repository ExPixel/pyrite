use std::sync::Arc;

use crate::gba_runner::SharedGba;
use eframe::{
    egui_glow::{CallbackFn, Painter},
    glow::{self, Buffer, HasContext, Program, Shader, Texture, VertexArray},
};
use egui::PaintCallbackInfo;
use parking_lot::Mutex;

pub struct GbaImageGlow {
    glow_painter: Arc<Mutex<GlowPainter>>,
    callback: Arc<CallbackFn>,
}

impl GbaImageGlow {
    pub fn new(gba: SharedGba) -> anyhow::Result<Self> {
        let glow_painter = Arc::new(Mutex::new(GlowPainter::new(gba)));

        let callback = Arc::new({
            let glow_painter = glow_painter.clone();
            CallbackFn::new(move |info, painter| {
                glow_painter.lock().paint(info, painter);
            })
        });

        Ok(Self {
            glow_painter,
            callback,
        })
    }

    pub fn paint(&mut self, rect: egui::Rect) -> egui::PaintCallback {
        egui::PaintCallback {
            rect,
            callback: self.callback.clone(),
        }
    }

    pub fn destroy(&mut self, gl: &eframe::glow::Context) {
        self.glow_painter.lock().destroy(gl)
    }
}

struct GlowPainter {
    gba: SharedGba,
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
    program: Option<Program>,
    buffer: Option<Buffer>,
    vertex_array: Option<VertexArray>,
    texture: Option<Texture>,
    initialized: bool,
}

impl GlowPainter {
    fn new(gba: SharedGba) -> Self {
        Self {
            gba,
            vertex_shader: None,
            fragment_shader: None,
            program: None,
            buffer: None,
            vertex_array: None,
            texture: None,
            initialized: false,
        }
    }

    fn paint(&mut self, _info: PaintCallbackInfo, painter: &Painter) {
        if !self.initialized {
            if let Err(err) = self.init(painter.gl()) {
                tracing::error!(error = debug(&err), "error while initializing GBA screen");
                panic!("error while initializing GBA screen: {err}");
            }
        }

        let gl = painter.gl();
        unsafe {
            gl.bind_buffer(eframe::glow::ARRAY_BUFFER, self.buffer);
            gl.bind_vertex_array(self.vertex_array);
            gl.use_program(self.program);
            gl.active_texture(eframe::glow::TEXTURE0);
            gl.bind_texture(eframe::glow::TEXTURE_2D, self.texture);
        }

        let mut gba_data = self.gba.write();
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
    }

    fn init(&mut self, gl: &eframe::glow::Context) -> Result<(), String> {
        unsafe {
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER)?;
            gl.shader_source(vertex_shader, GL_VERT_SHADER_SRC);
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                return Err(gl.get_shader_info_log(vertex_shader));
            }
            self.vertex_shader = Some(vertex_shader);

            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER)?;
            gl.shader_source(fragment_shader, GL_FRAG_SHADER_SRC);
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                return Err(gl.get_shader_info_log(fragment_shader));
            }
            self.fragment_shader = Some(fragment_shader);

            let program = gl.create_program()?;
            gl.attach_shader(program, self.vertex_shader.unwrap());
            gl.attach_shader(program, self.fragment_shader.unwrap());
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }
            self.program = Some(program);
            tracing::debug!("GBA screen GL program linked");

            let buffer = gl.create_buffer()?;
            self.buffer = Some(buffer);
            gl.bind_buffer(glow::ARRAY_BUFFER, self.buffer);
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice::<_, u8>(&GL_DEFAULT_VERTICES),
                glow::STATIC_DRAW,
            );
            tracing::debug!("GBA screen vertex buffer initialized");

            let vertex_array = gl.create_vertex_array()?;
            self.vertex_array = Some(vertex_array);
            gl.bind_vertex_array(self.vertex_array);
            let sz_float = std::mem::size_of::<f32>() as i32;
            let pos = gl
                .get_attrib_location(program, "in_position")
                .expect("no in_position attribute");
            let tex = gl
                .get_attrib_location(program, "in_texcoord")
                .expect("no in_texcoord attribute");
            gl.vertex_attrib_pointer_f32(pos, 2, eframe::glow::FLOAT, false, 4 * sz_float, 0);
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
            tracing::debug!("GBA screen vertex array object initialized");

            let sz_float = std::mem::size_of::<f32>() as i32;
            let pos = gl
                .get_attrib_location(program, "in_position")
                .expect("no in_position attribute");
            let tex = gl
                .get_attrib_location(program, "in_texcoord")
                .expect("no in_texcoord attribute");
            gl.vertex_attrib_pointer_f32(pos, 2, eframe::glow::FLOAT, false, 4 * sz_float, 0);
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

            let texture = gl.create_texture()?;
            self.texture = Some(texture);
            gl.bind_texture(glow::TEXTURE_2D, self.texture);

            let mut gba_data = self.gba.write();
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
        }

        self.initialized = true;
        Ok(())
    }

    fn destroy(&mut self, gl: &eframe::glow::Context) {
        if let Some(program) = self.program.take() {
            unsafe { gl.delete_program(program) };
        }

        if let Some(fragment_shader) = self.fragment_shader.take() {
            unsafe { gl.delete_shader(fragment_shader) };
        }

        if let Some(vertex_shader) = self.vertex_shader.take() {
            unsafe { gl.delete_shader(vertex_shader) };
        }

        if let Some(buffer) = self.buffer.take() {
            unsafe { gl.delete_buffer(buffer) };
        }

        if let Some(vertex_array) = self.vertex_array.take() {
            unsafe { gl.delete_vertex_array(vertex_array) };
        }

        if let Some(texture) = self.texture.take() {
            unsafe { gl.delete_texture(texture) };
        }

        self.initialized = false;
    }
}

const GL_FRAG_SHADER_SRC: &str = "\
#version 150 core
in vec2 frag_texcoord;
out vec4 out_color;
uniform sampler2D tex;
void main() {
    vec4 col = texture(tex, frag_texcoord);
    out_color = vec4(col.rgb, 1.0);
}";

const GL_VERT_SHADER_SRC: &str = "\
#version 150 core
in vec2 in_position;
in vec2 in_texcoord;
out vec2 frag_texcoord;
void main() {
    gl_Position = vec4(in_position, 0.0, 1.0);
    frag_texcoord = in_texcoord;
}";

#[rustfmt::skip]
const GL_DEFAULT_VERTICES: [f32; 24] = [
    -1.0,  1.0, 0.0, 0.0, // left, top
     1.0,  1.0, 1.0, 0.0, // right, top
    -1.0, -1.0, 0.0, 1.0, // left, bottom
    -1.0, -1.0, 0.0, 1.0, // left, bottom
     1.0, -1.0, 1.0, 1.0, // right, bottom
     1.0,  1.0, 1.0, 0.0, // right, top
];
