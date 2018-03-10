extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath;
extern crate cgmath_geometry;
extern crate glutin;

extern crate num_traits;

use gullery::ContextState;
use gullery::buffers::*;
use gullery::framebuffer::*;
use gullery::program::*;
use gullery::vao::*;
use gullery::glsl::*;
use gullery::colors::*;
use gullery::render_state::*;

use cgmath_geometry::OffsetBox;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlProfile, GlRequest};

#[derive(TypeGroup, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    color: Rgb<Nu8>
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms {
    offset: Point2<u32>
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(512, 512),
        ContextBuilder::new()
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0)
            })
            .with_srgb(true),
        &events_loop
    ).unwrap();
    unsafe{ window.context().make_current().unwrap() };
    let state = unsafe{ ContextState::new(|addr| window.context().get_proc_address(addr)) };

    let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        Vertex {
            pos: Vector2::new(-1.0, -1.0),
            color: Rgb::new(Nu8(255), Nu8(0), Nu8(0))
        },
        Vertex {
            pos: Vector2::new( 0.0,  1.0),
            color: Rgb::new(Nu8(0), Nu8(255), Nu8(0))
        },
        Vertex {
            pos: Vector2::new( 1.0,  -1.0),
            color: Rgb::new(Nu8(0), Nu8(0), Nu8(255))
        },
    ], state.clone());
    let vao = VertexArrayObj::new_noindex(vertex_buffer);


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let program = Program::new(&vertex_shader, None, &fragment_shader).unwrap_discard();

    let mut render_state = RenderState {
        srgb: true,
        ..RenderState::default()
    };

    let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(size_x, size_y) => {
                    let uniform = TriUniforms {
                        offset: Point2::new(0, 0)
                    };
                    render_state.viewport = OffsetBox {
                        origin: Point2::new(0, 0),
                        dims: Vector2::new(size_x, size_y)
                    };
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);

                    window.context().swap_buffers().unwrap();
                }
                WindowEvent::Closed => return ControlFlow::Break,
                _ => ()
            },
            _ => ()
        }

        ControlFlow::Continue
    });
}

const VERTEX_SHADER: &str = r#"
    #version 330

    in vec2 pos;
    in vec3 color;
    out vec3 vert_color;

    uniform uvec2 offset;

    void main() {
        vert_color = color;
        gl_Position = vec4(pos + vec2(offset), 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec3 vert_color;
    out vec4 frag_color;

    void main() {
        frag_color = vec4(vert_color, 1.0);
    }
"#;
