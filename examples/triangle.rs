extern crate gl_raii;
#[macro_use]
extern crate gl_raii_macros;
extern crate cgmath;
extern crate glutin;

use gl_raii::ContextState;
use gl_raii::buffers::*;
use gl_raii::framebuffer::*;
use gl_raii::program::*;
use gl_raii::vao::*;
use gl_raii::glsl::*;
use gl_raii::colors::*;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow};

#[derive(TypeGroup, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    matrix: Matrix2<f32>,
    color: Vector4<Nu8>,
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(512, 512),
        ContextBuilder::new(),
        &events_loop
    ).unwrap();
    unsafe{ window.context().make_current().unwrap() };
    let state = unsafe{ ContextState::new(|addr| window.context().get_proc_address(addr)) };

    // let matrix = Matrix3::new(0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    let matrix = Matrix2::new(0.5, 0.0, 0.0, 1.0);

    let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        Vertex {
            pos: Vector2::new(-1.0, -1.0),
            matrix,
            color: Vector4::new(Nu8(255), Nu8(255), Nu8(255), Nu8(255))
        },
        Vertex {
            pos: Vector2::new( 0.0,  1.0),
            matrix,
            color: Vector4::new(Nu8(255), Nu8(128), Nu8(255), Nu8(255))
        },
        Vertex {
            pos: Vector2::new( 1.0,  -1.0),
            matrix,
            color: Vector4::new(Nu8(0), Nu8(255), Nu8(255), Nu8(255))
        },
    ], state.clone()).unwrap();
    let vao = VertexArrayObj::new_noindex(vertex_buffer);

    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let program: Program<Vertex, ()> = Program::new(&vertex_shader, None, &fragment_shader).unwrap_werr();

    let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(_, _) => {
                    default_framebuffer.clear_color(RGBAf32::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program);

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
    in vec4 color;
    in mat2 matrix;
    out vec3 vert_color;

    void main() {
        gl_Position = vec4(matrix * pos, 0.0, 1.0);
        vert_color = color.rgb;
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

