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
use gullery::texture::types::ArrayTex;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::{cgmath, D2};
use cgmath_geometry::rect::{DimsBox, OffsetBox};

use std::io;
use std::fs::File;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlRequest, ElementState, VirtualKeyCode};
use glutin::dpi::LogicalSize;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    tex_coord: Vector2<u16>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, ArrayTex<ImageFormat<ScalarType=GLSLFloat>>>,
    array_index: u32
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
        WindowBuilder::new().with_dimensions(LogicalSize::new(512.0, 512.0)),
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
    let (ferris_normal_image, ferris_normal_dims) = load_image_from_file("./examples/textures/ferris_normal.png").unwrap();
    let (ferris_gesture_image, ferris_gesture_dims) = load_image_from_file("./examples/textures/ferris_gesture.png").unwrap();
    let (ferris_happy_image, ferris_happy_dims) = load_image_from_file("./examples/textures/ferris_happy.png").unwrap();
    assert_eq!(ferris_happy_dims, ferris_gesture_dims);
    assert_eq!(ferris_happy_dims, ferris_normal_dims);
    let mut image_combined = Vec::new();
    image_combined.extend_from_slice(&ferris_normal_image);
    image_combined.extend_from_slice(&ferris_gesture_image);
    image_combined.extend_from_slice(&ferris_happy_image);

    println!("texture loaded");
    let ferris_texture: Texture<D2, ArrayTex<SRgba>> = Texture::with_images(
        DimsBox::new3(ferris_happy_dims.width(), ferris_happy_dims.height(), 3),
        Some(SRgba::slice_from_raw(&image_combined)),
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
        ..RenderState::default()
    };

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    let mut array_index = 0;
    let mut redraw = |array_index| {
        let physical_size = window.get_inner_size().unwrap().to_physical(window.get_hidpi_factor());
        render_state.viewport = OffsetBox::new2(0, 0, physical_size.width as u32, physical_size.height as u32);
        let uniform = Uniforms {
            tex: ferris_texture.as_dyn(),
            array_index
        };
        default_framebuffer.clear_depth(1.0);
        default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
        default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);

        window.swap_buffers().unwrap();
    };

    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(_) => {
                    redraw(array_index);
                },
                WindowEvent::KeyboardInput{input, ..}
                    if input.state == ElementState::Pressed &&
                       input.virtual_keycode == Some(VirtualKeyCode::Space)
                => {
                    array_index += 1;
                    array_index %= ferris_texture.dims().depth();
                    println!("change array index to {}", array_index);
                    redraw(array_index);
                },

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

    uniform sampler2DArray tex;
    uniform uint array_index;

    void main() {
        color = texture(tex, vec3(tc, array_index));
    }
"#;
