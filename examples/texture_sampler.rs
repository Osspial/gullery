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
use gullery::texture::{*, sample_parameters::*};
use gullery::texture::targets::SimpleTex;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::cgmath;
use cgmath_geometry::{OffsetBox, DimsBox};

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlRequest, ElementState, VirtualKeyCode};

use std::{io, fs::File};
use std::rc::Rc;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    tex_coord: Vector2<u16>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: SampledTexture<'a, SimpleTex<SRgba, DimsBox<Point2<u32>>>, ()>,
    offset: Vector2<f32>,
    scale: Vector2<f32>
}

fn load_texture_from_file(path: &str, state: &Rc<ContextState>) -> Result<Texture<SimpleTex<SRgba, DimsBox<Point2<u32>>>, ()>, io::Error> {
    let (image, dims) = {
        let decoder = png::Decoder::new(File::open(path)?);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;
        (buf, DimsBox::new2(info.width, info.height))
    };
    println!("texture loaded: {}", path);
    let texture = Texture::with_images(
        dims,
        Some(SRgba::slice_from_raw(&image)),
        state.clone()
    )?;
    println!("texture uploaded: {}", path);
    Ok(texture)
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
    let ferris_normal_texture = load_texture_from_file("./examples/textures/ferris.png", &state).unwrap();
    let ferris_happy_texture = load_texture_from_file("./examples/textures/ferris_happy.png", &state).unwrap();
    let mut sampler = Sampler::new(state.clone());
    sampler.sample_parameters.filter_mag = FilterMag::Nearest;


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

    let anisotropy_values = [1.0, 2.0, 4.0, 8.0, 16.0];
    let mut anisotropy_index = 0;

    events_loop.run_forever(|event| {
        let mut redraw = |anisotropy_index| {
            render_state.viewport = OffsetBox::new2(0, 0, window.get_inner_size().unwrap().0, window.get_inner_size().unwrap().1);
            default_framebuffer.clear_depth(1.0);
            default_framebuffer.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));

            sampler.sample_parameters.anisotropy_max = anisotropy_values[anisotropy_index];
            let mut uniform = Uniforms {
                tex: SampledTexture {
                    texture: &ferris_normal_texture,
                    sampler: &sampler
                },
                offset: Vector2::new(-0.5, 0.5),
                scale: Vector2::new(0.5, 0.5)
            };

            let mut draw_scaled_copies = |mut uniform: Uniforms| {
                let copies = 6;

                for _ in 0..copies {
                    default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);
                    uniform.offset.y -= uniform.scale.y * 1.5;
                    uniform.scale.y /= 2.0;
                }
                println!();
            };

            draw_scaled_copies(uniform);

            uniform.tex.texture = &ferris_happy_texture;
            uniform.offset.x = 0.5;
            draw_scaled_copies(uniform);

            window.context().swap_buffers().unwrap();
        };

        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(size_x, size_y) => {
                    window.context().resize(size_x, size_y);
                    redraw(anisotropy_index);
                }
                WindowEvent::KeyboardInput{input, ..}
                    if input.state == ElementState::Pressed &&
                       input.virtual_keycode == Some(VirtualKeyCode::Space)
                => {
                    anisotropy_index += 1;
                    anisotropy_index %= anisotropy_values.len();
                    println!("Changed anisotropy to: {}", anisotropy_values[anisotropy_index]);
                    redraw(anisotropy_index)
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

    uniform vec2 offset;
    uniform vec2 scale;

    void main() {
        tc = tex_coord;
        gl_Position = vec4((pos * scale) + offset, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec2 tc;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = vec4(texture(tex, tc).xyz, 1);
    }
"#;
