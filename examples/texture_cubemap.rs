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
use gullery::image_format::{ImageFormat, SRgb, Rgba, compressed::DXT1};
use gullery::texture::*;
use gullery::texture::types::{CubemapImage, CubemapTex};
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::{cgmath, D2};
use cgmath_geometry::rect::{DimsBox, OffsetBox};

use std::io::BufReader;
use std::fs::File;

use cgmath::*;

use glutin::{GlContext, EventsLoop, Event, WindowEvent, ControlFlow, WindowBuilder, ContextBuilder, GlWindow, GlRequest, ElementState, VirtualKeyCode};
use glutin::dpi::LogicalSize;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Point3<f32>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, CubemapTex<ImageFormat<ScalarType=GLSLFloat>>>,
    matrix: Matrix4<f32>,
}

fn load_image_from_file(path: &str) -> (Vec<u8>, DimsBox<D2, u32>) {
    let mut file = BufReader::new(File::open(path).unwrap());
    let dds = ddsfile::Dds::read(&mut file).unwrap();
    let buf_len = dds.header.linear_size.unwrap() as usize;
    (dds.data[..buf_len].to_vec(), DimsBox::new2(dds.header.width, dds.header.height))
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
            pos: Point3::new(1.0, -1.0, -1.0)
        },
        Vertex {
            pos: Point3::new(1.0, -1.0, 1.0)
        },
        Vertex {
            pos: Point3::new(-1.0, -1.0, 1.0)
        },
        Vertex {
            pos: Point3::new(-1.0, -1.0, -1.0)
        },
        Vertex {
            pos: Point3::new(1.0, 1.0, -1.0)
        },
        Vertex {
            pos: Point3::new(1.0, 1.0, 1.0)
        },
        Vertex {
            pos: Point3::new(-1.0, 1.0, 1.0)
        },
        Vertex {
            pos: Point3::new(-1.0, 1.0, -1.0)
        },
    ], state.clone());
    let index_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        0, 3, 2,
        0, 2, 1,
        4, 5, 6,
        4, 6, 7,
        0, 1, 5,
        0, 5, 4,
        1, 2, 6,
        1, 6, 5,
        2, 3, 7,
        2, 7, 6,
        4, 7, 3,
        4, 3, 0u16,
    ], state.clone());
    let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));
    println!("vao created");
    let (pos_x, pos_x_dims) = load_image_from_file("./examples/textures/cubemap/pos_x.dds");
    let (pos_y, pos_y_dims) = load_image_from_file("./examples/textures/cubemap/pos_y.dds");
    let (pos_z, pos_z_dims) = load_image_from_file("./examples/textures/cubemap/pos_z.dds");
    let (neg_x, neg_x_dims) = load_image_from_file("./examples/textures/cubemap/neg_x.dds");
    let (neg_y, neg_y_dims) = load_image_from_file("./examples/textures/cubemap/neg_y.dds");
    let (neg_z, neg_z_dims) = load_image_from_file("./examples/textures/cubemap/neg_z.dds");
    assert_eq!(pos_x_dims.width(), pos_x_dims.height());
    assert_eq!(pos_x_dims, pos_y_dims);
    assert_eq!(pos_y_dims, pos_z_dims);
    assert_eq!(pos_z_dims, neg_x_dims);
    assert_eq!(neg_x_dims, neg_y_dims);
    assert_eq!(neg_y_dims, neg_z_dims);

    println!("texture loaded");
    let ferris_texture: Texture<D2, CubemapTex<DXT1<SRgb>>> = Texture::with_images(
        DimsSquare::new(pos_x_dims.width()),
        Some(CubemapImage {
            pos_x: DXT1::slice_from_raw(&pos_x),
            pos_y: DXT1::slice_from_raw(&pos_y),
            pos_z: DXT1::slice_from_raw(&pos_z),
            neg_x: DXT1::slice_from_raw(&neg_x),
            neg_y: DXT1::slice_from_raw(&neg_y),
            neg_z: DXT1::slice_from_raw(&neg_z),
        }),
        state.clone()
    ).unwrap();
    println!("vao created");


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, warning) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();
    for w in warning {
        println!("{:?}", w);
    }

    let mut render_state = RenderState {
        srgb: true,
        cull: Some((CullFace::Front, FrontFace::Clockwise)),
        viewport: OffsetBox {
            origin: Point2::new(0, 0),
            dims: Vector2::new(512, 512)
        },
        ..RenderState::default()
    };

    let z_near = 0.1;
    let z_far = 10.0;
    let fov: f32 = 70.0;

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    let mut rotation = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));

    let mut redraw = |rotation, aspect_ratio| {
        let physical_size = window.get_inner_size().unwrap().to_physical(window.get_hidpi_factor());
        render_state.viewport = OffsetBox::new2(0, 0, physical_size.width as u32, physical_size.height as u32);
        let scale = 1.0 / (fov.to_radians() / 2.0).tan();
        let perspective_matrix = Matrix4::new(
                scale / aspect_ratio, 0.0  , 0.0                                      , 0.0,
                0.0                 , scale, 0.0                                      , 0.0,
                0.0                 , 0.0  , (z_near + z_far) / (z_near - z_far)      , -1.0,
                0.0                 , 0.0  , (2.0 * z_far * z_near) / (z_near - z_far), 0.0
            );
        let uniform = Uniforms {
            tex: ferris_texture.as_dyn(),
            matrix: perspective_matrix * Matrix4::from(Matrix3::from(Basis3::from(Quaternion::from(rotation)))),
        };
        default_framebuffer.clear_depth(1.0);
        default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
        default_framebuffer.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);

        window.swap_buffers().unwrap();
    };

    let mut aspect_ratio = 1.0;
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Resized(d) => {
                    aspect_ratio = (d.width / d.height) as f32;
                    redraw(rotation, aspect_ratio);
                },
                WindowEvent::KeyboardInput{input, ..}
                    if input.state == ElementState::Pressed
                => {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::A) => rotation.y.0 -= 1.0,
                        Some(VirtualKeyCode::D) => rotation.y.0 += 1.0,
                        Some(VirtualKeyCode::W) => rotation.x.0 -= 1.0,
                        Some(VirtualKeyCode::S) => rotation.x.0 += 1.0,
                        _ => (),
                    }
                    redraw(rotation, aspect_ratio);
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

    in vec3 pos;
    uniform mat4 matrix;
    out vec3 tc;

    void main() {
        tc = pos;
        gl_Position = matrix * vec4(pos, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec3 tc;
    out vec4 color;

    uniform samplerCube tex;

    void main() {
        color = texture(tex, tc);
    }
"#;

