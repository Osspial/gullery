#![feature(never_type)]

extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath_geometry;
extern crate glutin;
extern crate png;

extern crate num_traits;

use gullery::ContextState;
use gullery::buffer::*;
use gullery::framebuffer::{*, render_state::*};
use gullery::program::*;
use gullery::image_format::*;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::cgmath;
use cgmath_geometry::rect::OffsetBox;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlProfile, GlRequest};
use glutin::dpi::LogicalSize;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    color: Rgb<u8>
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms {
    offset: Point2<f32>
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(LogicalSize::new(512.0, 512.0)),
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
            color: Rgb::new(255, 0, 0)
        },
        Vertex {
            pos: Vector2::new( 0.0,  1.0),
            color: Rgb::new(0, 255, 0)
        },
        Vertex {
            pos: Vector2::new( 1.0,  -1.0),
            color: Rgb::new(0, 0, 255)
        },
    ], state.clone());
    let vao = VertexArrayObject::<_, !>::new(vertex_buffer, None);


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

    let mut render_state = RenderState {
        srgb: true,
        viewport: OffsetBox {
            origin: Point2::new(0, 0),
            dims: Vector2::new(512, 512)
        },
        ..RenderState::default()
    };

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(logical_size) => {
                    let physical_size = logical_size.to_physical(window.get_hidpi_factor());
                    window.resize(physical_size);
                    let uniform = TriUniforms {
                        offset: Point2::new(0.0, 0.0)
                    };
                    render_state.viewport = OffsetBox::new2(0, 0, physical_size.width as _, physical_size.height as _);
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);

                    window.swap_buffers().unwrap();
                }
                WindowEvent::CloseRequested => return ControlFlow::Break,
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

    uniform vec2 offset;

    void main() {
        vert_color = color;
        gl_Position = vec4(pos + vec2(offset), 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec3 vert_color;
    out vec4 color;

    void main() {
        color = vec4(vert_color, 1.0);
    }
"#;
