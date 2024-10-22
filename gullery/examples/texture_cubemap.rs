extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath;
extern crate glutin;
extern crate png;

mod helper;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    geometry::{GLMat4r4c, GLSLFloat, GLVec2, GLVec3},
    image_format::{compressed::DXT1, ImageFormat, Rgba, SRgb},
    program::*,
    texture::{
        types::{CubemapImage, CubemapTex},
        *,
    },
    vertex::VertexArrayObject,
    ContextState,
};

use cgmath::{Basis3, Deg, Euler, Matrix3, Matrix4, Quaternion};

use glutin::{dpi::LogicalSize, *};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec3<f32>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, CubemapTex<dyn ImageFormat<ScalarType = GLSLFloat>>>,
    matrix: GLMat4r4c<f32>,
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(LogicalSize::new(512.0, 512.0)),
        ContextBuilder::new()
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 3),
                opengles_version: (3, 0),
            })
            .with_srgb(true),
        &events_loop,
    )
    .unwrap();
    unsafe { window.context().make_current().unwrap() };
    let state = unsafe { ContextState::new(|addr| window.context().get_proc_address(addr)) };

    let vertex_buffer = Buffer::with_data(
        BufferUsage::StaticDraw,
        &[
            Vertex {
                pos: GLVec3::new(-1.0, -1.0, -1.0),
            },
            Vertex {
                pos: GLVec3::new(-1.0, -1.0, 1.0),
            },
            Vertex {
                pos: GLVec3::new(1.0, -1.0, 1.0),
            },
            Vertex {
                pos: GLVec3::new(1.0, -1.0, -1.0),
            },
            Vertex {
                pos: GLVec3::new(-1.0, 1.0, -1.0),
            },
            Vertex {
                pos: GLVec3::new(-1.0, 1.0, 1.0),
            },
            Vertex {
                pos: GLVec3::new(1.0, 1.0, 1.0),
            },
            Vertex {
                pos: GLVec3::new(1.0, 1.0, -1.0),
            },
        ],
        state.clone(),
    );
    let index_buffer = Buffer::with_data(
        BufferUsage::StaticDraw,
        &[
            0, 3, 2, 0, 2, 1, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7,
            6, 4, 7, 3, 4, 3, 0u16,
        ],
        state.clone(),
    );
    let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));
    println!("vao created");
    let (pos_x_mips, pos_x_dims) = helper::load_dxt1_from_dds("./textures/cubemap/pos_x.dds");
    let (pos_y_mips, pos_y_dims) = helper::load_dxt1_from_dds("./textures/cubemap/pos_y.dds");
    let (pos_z_mips, pos_z_dims) = helper::load_dxt1_from_dds("./textures/cubemap/pos_z.dds");
    let (neg_x_mips, neg_x_dims) = helper::load_dxt1_from_dds("./textures/cubemap/neg_x.dds");
    let (neg_y_mips, neg_y_dims) = helper::load_dxt1_from_dds("./textures/cubemap/neg_y.dds");
    let (neg_z_mips, neg_z_dims) = helper::load_dxt1_from_dds("./textures/cubemap/neg_z.dds");
    assert_eq!(pos_x_dims.width(), pos_x_dims.height());
    assert_eq!(pos_x_dims, pos_y_dims);
    assert_eq!(pos_y_dims, pos_z_dims);
    assert_eq!(pos_z_dims, neg_x_dims);
    assert_eq!(neg_x_dims, neg_y_dims);
    assert_eq!(neg_y_dims, neg_z_dims);

    println!("texture loaded");
    let mips = (0..pos_x_mips.len()).map(|i| CubemapImage {
        pos_x: &pos_x_mips[i],
        pos_y: &pos_y_mips[i],
        pos_z: &pos_z_mips[i],
        neg_x: &neg_x_mips[i],
        neg_y: &neg_y_mips[i],
        neg_z: &neg_z_mips[i],
    });
    let cubemap_texture: Texture<D2, CubemapTex<DXT1<SRgb>>> =
        Texture::with_images(DimsSquare::new(pos_x_dims.width()), mips, state.clone()).unwrap();
    println!("vao created");

    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, warning) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();
    for w in warning {
        println!("{:?}", w);
    }

    let mut render_state = RenderState {
        srgb: true,
        texture_cubemap_seamless: true,
        cull: Some((CullFace::Front, FrontFace::CounterClockwise)),
        viewport: GLVec2::new(0, 0)..=GLVec2::new(512, 512),
        ..RenderState::default()
    };

    let z_near = 0.1;
    let z_far = 10.0;
    let fov: f32 = 70.0;

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    let mut rotation = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));

    let mut redraw = |rotation, aspect_ratio, use_perspective| {
        let physical_size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        render_state.viewport = GLVec2::new(0, 0)
            ..=GLVec2::new(physical_size.width as u32, physical_size.height as u32);
        let scale = 1.0 / (fov.to_radians() / 2.0).tan();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let perspective_matrix = match use_perspective {
            true => Matrix4::new(
                scale / aspect_ratio, 0.0,   0.0,                                       0.0,
                0.0,                  scale, 0.0,                                       0.0,
                0.0,                  0.0,   (z_near + z_far) / (z_near - z_far),      -1.0,
                0.0,                  0.0,   (2.0 * z_far * z_near) / (z_near - z_far), 0.0,
            ),
            false => Matrix4::new(
                scale / aspect_ratio, 0.0, 0.0, 0.0,
                0.0, scale, 0.0, 0.0,
                0.0, 0.0, (z_near + z_far) / (z_near - z_far), -1.0,
                0.0, 0.0, 0.0, 1.0,
            ),
        };
        let uniform = Uniforms {
            tex: cubemap_texture.as_dyn(),
            matrix: GLMat4r4c::from(
                perspective_matrix
                    * Matrix4::from(Matrix3::from(Basis3::from(Quaternion::from(rotation)))),
            ),
        };
        default_framebuffer.clear_depth(1.0);
        default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
        default_framebuffer.draw(
            DrawMode::Triangles,
            ..,
            &vao,
            &program,
            &uniform,
            &render_state,
        );

        window.swap_buffers().unwrap();
    };

    let mut aspect_ratio = 1.0;
    let mut window_focused = true;
    let mouse_sensitivity = 0.1;
    let mut use_perspective = true;
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(d) => {
                    aspect_ratio = (d.width / d.height) as f32;
                    redraw(rotation, aspect_ratio, use_perspective);
                }
                WindowEvent::Focused(f) => {
                    window_focused = f;
                    if f {
                        window.grab_cursor(true).ok();
                        window.hide_cursor(true);
                    } else {
                        window.grab_cursor(false).ok();
                        window.hide_cursor(false);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    window.grab_cursor(true).ok();
                    window.hide_cursor(true);
                    window_focused = true;
                }
                WindowEvent::KeyboardInput { input, .. }
                    if input.state == ElementState::Pressed =>
                {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => {
                            window.grab_cursor(false).ok();
                            window.hide_cursor(false);
                            window_focused = false;
                        }
                        Some(VirtualKeyCode::Space) => {
                            use_perspective = !use_perspective;
                        }
                        _ => (),
                    }
                    redraw(rotation, aspect_ratio, use_perspective);
                }
                WindowEvent::CloseRequested => return ControlFlow::Break,
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } if window_focused => {
                    rotation.x.0 += delta.1 as f32 * mouse_sensitivity;
                    rotation.y.0 += delta.0 as f32 * mouse_sensitivity;
                    redraw(rotation, aspect_ratio, use_perspective);
                }
                _ => (),
            },
            _ => (),
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
