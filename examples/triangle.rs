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
use gl_raii::textures::*;

use cgmath::*;
use std::iter;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow};

#[derive(TypeGroup, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms<'a> {
    texture: &'a Texture<Rgb<Nu8>, targets::SimpleTex<Dims2D>>
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

    let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        Vertex {
            pos: Vector2::new(-1.0, -1.0),
        },
        Vertex {
            pos: Vector2::new( 0.0,  1.0),
        },
        Vertex {
            pos: Vector2::new( 1.0,  -1.0),
        },
    ], state.clone()).unwrap();
    let vao = VertexArrayObj::new_noindex(vertex_buffer);


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let program = Program::new(&vertex_shader, None, &fragment_shader).unwrap_discard();

    let mut image = Vec::new();
    for i in 0..512u32*512 {
        image.push(Rgb::new(Nu8(255), Nu8(255), Nu8((i % 255) as u8)));
    }
    let texture = Texture::with_images(Dims2D::new(512, 512), iter::once(&image[..]), state.clone()).unwrap();

    let uniforms = TriUniforms {
        texture: &texture
    };

    let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(_, _) => {
                    default_framebuffer.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniforms);

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
    out vec2 tex_coord;

    void main() {
        tex_coord = pos;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec2 tex_coord;
    out vec4 frag_color;

    uniform sampler2D tex;

    void main() {
        frag_color = texture(tex, tex_coord);
    }
"#;
