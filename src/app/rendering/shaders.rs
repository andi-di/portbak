//! Screen-space shader effects on top of egui's OpenGL renderer. egui cannot sample what it has
//! drawn, so each effect is a paint callback that copies the pixels under it into a texture and
//! draws them back through a fragment shader. Programs are built on first use; if the GPU turns
//! one down the effect stays off and callers keep their plain look.

use eframe::egui::{self, Pos2, Rect};
use eframe::egui_glow;
use eframe::glow::{self, HasContext};
use std::sync::{Arc, Mutex};

/// A quad covering the callback's viewport, made from the vertex index so no buffer is needed.
const VERTEX_SHADER: &str = r#"
out vec2 v_uv;
void main() {
    v_uv = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
    gl_Position = vec4(v_uv * 2.0 - 1.0, 0.0, 1.0);
}
"#;

/// Inputs every fragment shader gets: the copied frame, its size in pixels and the quad
/// position. The copy sits in the corner of a screen-sized texture, so shaders read it through
/// `frame`, which also keeps them from sampling past the copy's edge.
const FRAGMENT_HEADER: &str = r#"
uniform sampler2D u_frame;
uniform vec2 u_size;
// One over the whole texture's size in pixels.
uniform vec2 u_texel;
in vec2 v_uv;
out vec4 out_color;

vec4 frame(vec2 pixel) {
    return texture(u_frame, clamp(pixel, vec2(0.5), u_size - 0.5) * u_texel);
}
"#;

/// One effect: a named fragment shader.
pub struct Recipe {
    pub name: &'static str,
    /// The shader body after `FRAGMENT_HEADER`.
    pub fragment: &'static str,
}

/// GL objects made on the render thread the first time the effect draws.
struct Gpu {
    program: glow::Program,
    vertices: glow::VertexArray,
    frame: glow::Texture,
    /// Size of the frame texture's storage, reallocated only when the window outgrows it.
    storage: [i32; 2],
}

#[derive(Default)]
enum State {
    #[default]
    Pending,
    Ready(Gpu),
    Failed,
}

/// Handle to an effect's GL state, shared between frames and the render callbacks.
#[derive(Clone)]
pub struct Effect {
    recipe: &'static Recipe,
    state: Arc<Mutex<State>>,
}

/// What the effect's uniforms are set through while it draws.
pub struct Target<'a> {
    gl: &'a glow::Context,
    program: glow::Program,
    info: &'a egui::PaintCallbackInfo,
}

impl Target<'_> {
    /// `point` in the frame copy's pixels, whose origin is its bottom-left corner.
    pub fn pixel(&self, point: Pos2) -> [f32; 2] {
        let viewport = self.info.viewport_in_pixels();
        let scale = self.info.pixels_per_point;
        [
            point.x * scale - viewport.left_px as f32,
            self.info.screen_size_px[1] as f32 - point.y * scale - viewport.from_bottom_px as f32,
        ]
    }

    /// `points` in pixels.
    pub fn pixels(&self, points: f32) -> f32 {
        points * self.info.pixels_per_point
    }

    pub fn set(&self, name: &str, values: &[f32]) {
        unsafe {
            let location = self.gl.get_uniform_location(self.program, name);
            let location = location.as_ref();
            match *values {
                [x] => self.gl.uniform_1_f32(location, x),
                [x, y] => self.gl.uniform_2_f32(location, x, y),
                [x, y, z] => self.gl.uniform_3_f32(location, x, y, z),
                [x, y, z, w] => self.gl.uniform_4_f32(location, x, y, z, w),
                _ => {}
            }
        }
    }
}

impl Effect {
    /// The effect for `recipe` on this context, shared by every frame.
    pub fn of(ctx: &egui::Context, recipe: &'static Recipe) -> Self {
        let id = egui::Id::new(("aero_shader", recipe.name));
        ctx.data_mut(|data| {
            data.get_temp_mut_or_insert_with(id, || Self {
                recipe,
                state: Arc::default(),
            })
            .clone()
        })
    }

    /// False once the GPU turned the shader down, so callers can draw something simpler.
    pub fn available(&self) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| !matches!(*state, State::Failed))
    }

    /// A shape that redraws the pixels under `rect` through the shader, with uniforms from
    /// `uniforms`; nothing where `rect` lies off `screen`.
    pub fn shape(
        &self,
        rect: Rect,
        screen: Rect,
        uniforms: impl Fn(&Target) + Send + Sync + 'static,
    ) -> egui::Shape {
        let rect = rect.intersect(screen);
        if !rect.is_positive() {
            return egui::Shape::Noop;
        }
        let effect = self.clone();
        let callback = egui_glow::CallbackFn::new(move |info, painter| {
            effect.draw(painter.gl(), &info, &uniforms);
        });
        egui::Shape::Callback(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        })
    }

    fn draw(&self, gl: &glow::Context, info: &egui::PaintCallbackInfo, uniforms: &dyn Fn(&Target)) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if matches!(*state, State::Pending) {
            *state = match unsafe { Gpu::new(gl, self.recipe) } {
                Ok(gpu) => State::Ready(gpu),
                Err(error) => {
                    eprintln!("Shader {} unavailable: {error}", self.recipe.name);
                    State::Failed
                }
            };
        }
        if let State::Ready(gpu) = &mut *state {
            unsafe { gpu.draw(gl, info, uniforms) };
        }
    }
}

impl Gpu {
    unsafe fn new(gl: &glow::Context, recipe: &Recipe) -> Result<Self, String> {
        let version = egui_glow::ShaderVersion::get(gl);
        if !version.is_new_shader_interface() {
            return Err(format!("{version:?} is too old"));
        }
        let header = format!(
            "{}#ifdef GL_ES\nprecision highp float;\n#endif\n",
            version.version_declaration()
        );
        let program = gl.create_program()?;
        let mut shaders = Vec::new();
        for (kind, source) in [
            (glow::VERTEX_SHADER, VERTEX_SHADER.to_owned()),
            (
                glow::FRAGMENT_SHADER,
                format!("{FRAGMENT_HEADER}{}", recipe.fragment),
            ),
        ] {
            let shader = gl.create_shader(kind)?;
            gl.shader_source(shader, &format!("{header}{source}"));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                return Err(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            return Err(gl.get_program_info_log(program));
        }
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        let frame = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(frame));
        for (parameter, value) in [
            (glow::TEXTURE_MIN_FILTER, glow::LINEAR),
            (glow::TEXTURE_MAG_FILTER, glow::LINEAR),
            (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
            (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
        ] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, value as i32);
        }
        Ok(Self {
            program,
            vertices: gl.create_vertex_array()?,
            frame,
            storage: [0, 0],
        })
    }

    /// Copies the pixels under the callback's viewport and draws them back through the shader.
    unsafe fn draw(
        &mut self,
        gl: &glow::Context,
        info: &egui::PaintCallbackInfo,
        uniforms: &dyn Fn(&Target),
    ) {
        let viewport = info.viewport_in_pixels();
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.frame));
        let screen = info.screen_size_px.map(|side| side as i32);
        if self.storage[0] < screen[0] || self.storage[1] < screen[1] {
            self.storage = screen;
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                screen[0],
                screen[1],
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
        }
        gl.copy_tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            viewport.left_px,
            viewport.from_bottom_px,
            viewport.width_px,
            viewport.height_px,
        );
        gl.use_program(Some(self.program));
        let target = Target {
            gl,
            program: self.program,
            info,
        };
        gl.uniform_1_i32(gl.get_uniform_location(self.program, "u_frame").as_ref(), 0);
        target.set(
            "u_size",
            &[viewport.width_px as f32, viewport.height_px as f32],
        );
        target.set(
            "u_texel",
            &[1.0 / self.storage[0] as f32, 1.0 / self.storage[1] as f32],
        );
        uniforms(&target);
        gl.bind_vertex_array(Some(self.vertices));
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST: Recipe = Recipe {
        name: "test",
        fragment: "void main() { out_color = frame(v_uv * u_size); }",
    };

    #[test]
    fn effects_are_shared_per_context_and_clipped_to_the_screen() {
        let ctx = egui::Context::default();
        let effect = Effect::of(&ctx, &TEST);
        assert!(Arc::ptr_eq(&effect.state, &Effect::of(&ctx, &TEST).state));
        assert!(effect.available());
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let partly = Rect::from_min_size(egui::pos2(700.0, 500.0), egui::vec2(300.0, 300.0));
        let egui::Shape::Callback(callback) = effect.shape(partly, screen, |_| {}) else {
            panic!("an on-screen effect is a paint callback");
        };
        assert!(screen.contains_rect(callback.rect));
        let off = partly.translate(egui::vec2(2_000.0, 0.0));
        assert!(matches!(
            effect.shape(off, screen, |_| {}),
            egui::Shape::Noop
        ));
    }
}
