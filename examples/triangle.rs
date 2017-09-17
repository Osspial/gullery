extern crate gl_raii;
#[macro_use]
extern crate gl_raii_macros;
extern crate cgmath;
extern crate cgmath_geometry;
extern crate glutin;

extern crate num_traits;

use gl_raii::ContextState;
use gl_raii::buffers::*;
use gl_raii::framebuffer::*;
use gl_raii::program::*;
use gl_raii::vao::*;
use gl_raii::glsl::*;
use gl_raii::colors::*;
use gl_raii::textures::*;
use gl_raii::render_state::*;

use cgmath_geometry::OffsetRect;

use cgmath::*;

use num_traits::NumCast;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow};

#[derive(TypeGroup, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    color: Rgb<Nu8>
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms<'a> {
    tex: &'a Texture<Rgb<Nu8>, targets::SimpleTex<Dims2D>>
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(512, 512),
        ContextBuilder::new().with_multisampling(8).with_srgb(true),
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
    println!("{:?}", <Nu16 as NumCast>::from(Ni8(-64)));


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
                    render_state.viewport = OffsetRect {
                        origin: Point2::new(0, 0),
                        dims: Vector2::new(size_x, size_y)
                    };
                    default_framebuffer.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, (), render_state);

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

    void main() {
        vert_color = color;
        gl_Position = vec4(pos, 0.0, 1.0);
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
