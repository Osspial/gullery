extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath_geometry;
extern crate glutin;
extern crate png;

extern crate num_traits;

use gullery::ContextState;
use gullery::glsl::GLSLFloat;
use gullery::buffer::*;
use gullery::framebuffer::{*, render_state::*};
use gullery::program::*;
use gullery::image_format::*;
use gullery::texture::*;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::{cgmath, D2};
use cgmath_geometry::rect::{DimsBox, OffsetBox};

use std::io;
use std::fs::File;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlRequest};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    tex_coord: Vector2<u16>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, ImageFormat<ScalarType=GLSLFloat>>
}

fn load_image_from_file(path: &str) -> Result<(Vec<u8>, DimsBox<D2, u32>), io::Error> {
    let decoder = png::Decoder::new(File::open(path)?);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;
    Ok((buf, DimsBox::new2(info.width, info.height)))
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
    let images = [
        load_image_from_file("./examples/textures/mips/512.png").unwrap(),
        load_image_from_file("./examples/textures/mips/256.png").unwrap(),
        load_image_from_file("./examples/textures/mips/128.png").unwrap(),
        load_image_from_file("./examples/textures/mips/64.png").unwrap(),
        load_image_from_file("./examples/textures/mips/32.png").unwrap(),
        load_image_from_file("./examples/textures/mips/16.png").unwrap(),
        load_image_from_file("./examples/textures/mips/8.png").unwrap(),
    ];
    let dims = images[0].1;
    println!("texture loaded");
    let mip_texture: Texture<D2, Rgba> = Texture::with_images(
        dims,
        images.iter().map(|(mip, _)| Rgba::slice_from_raw(&mip[..])),
        state.clone()
    ).unwrap();
    println!("texture uploaded");


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

    let mut render_state = RenderState {
        srgb: true,
        viewport: OffsetBox {
            origin: Point2::new(0, 0),
            dims: Vector2::new(512, 512)
        },
        blend: Some(BlendFuncs {
            src_rgb: BlendFunc::SrcAlpha,
            dst_rgb: BlendFunc::OneMinusSrcAlpha,
            src_alpha: BlendFunc::One,
            dst_alpha: BlendFunc::Zero,
        }),
        ..RenderState::default()
    };

    let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(size_x, size_y) => {
                    window.context().resize(size_x, size_y);
                    let uniform = Uniforms {
                        tex: mip_texture.as_dyn()
                    };
                    render_state.viewport = OffsetBox::new2(0, 0, size_x, size_y);
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color(Rgba::new(1.0, 1.0, 1.0, 1.0));
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
