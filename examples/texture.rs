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
use gullery::color::*;
use gullery::texture::*;
use gullery::texture::targets::SimpleTex;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::cgmath;
use cgmath_geometry::{OffsetBox, DimsBox};

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlRequest};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    tex_coord: Vector2<u16>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<SimpleTex<SRgba, DimsBox<Point2<u32>>>>
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(512, 512),
        ContextBuilder::new()
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 3),
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
            tex_coord: Vector2::new(0, !0)
        },
        Vertex {
            pos: Vector2::new(-1.0,  1.0),
            tex_coord: Vector2::new(0, 0)
        },
        Vertex {
            pos: Vector2::new( 1.0,  1.0),
            tex_coord: Vector2::new(!0, 0)
        },
        Vertex {
            pos: Vector2::new( 1.0, -1.0),
            tex_coord: Vector2::new(!0, !0)
        },
    ], state.clone());
    let index_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        0, 1, 2,
        2, 3, 0u16
    ], state.clone());
    let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));
    println!("vao created");
    let (ferris_image, ferris_dims) = {
        use std::fs::File;
        let decoder = png::Decoder::new(File::open("./examples/ferris.png").unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        (buf, DimsBox::new2(info.width, info.height))
    };
    let ferris_texture = Texture::with_images(
        ferris_dims,
        Some(SRgba::slice_from_raw(&ferris_image)),
        state.clone()
    ).unwrap();
    println!("texture loaded");


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

    let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(size_x, size_y) => {
                    window.context().resize(size_x, size_y);
                    let uniform = Uniforms {
                        tex: &ferris_texture
                    };
                    render_state.viewport = OffsetBox::new2(0, 0, size_x, size_y);
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
    in vec2 tex_coord;
    out vec2 tc;

    void main() {
        tc = tex_coord;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec2 tc;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = texture(tex, tc);
    }
"#;
