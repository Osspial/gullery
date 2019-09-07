extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath_geometry;
extern crate glutin;
extern crate png;

extern crate num_traits;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    glsl::GLSLFloat,
    image_format::*,
    program::*,
    texture::*,
    vertex::VertexArrayObject,
    ContextState,
};

use cgmath_geometry::{
    cgmath,
    rect::{DimsBox, OffsetBox},
    D2,
};

use cgmath::*;

use glutin::{
    dpi::LogicalSize, ContextBuilder, ControlFlow, Event, EventsLoop, GlContext, GlRequest,
    GlWindow, WindowBuilder, WindowEvent,
};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    tex_coord: Vector2<u16>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, dyn ImageFormat<ScalarType = GLSLFloat>>,
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
                pos: Vector2::new(-1.0, -1.0),
                tex_coord: Vector2::new(0, !0),
            },
            Vertex {
                pos: Vector2::new(-1.0, 1.0),
                tex_coord: Vector2::new(0, 0),
            },
            Vertex {
                pos: Vector2::new(1.0, 1.0),
                tex_coord: Vector2::new(!0, 0),
            },
            Vertex {
                pos: Vector2::new(1.0, -1.0),
                tex_coord: Vector2::new(!0, !0),
            },
        ],
        state.clone(),
    );
    let index_buffer = Buffer::with_data(
        BufferUsage::StaticDraw,
        &[0, 1, 2, 2, 3, 0u16],
        state.clone(),
    );
    let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));
    println!("vao created");
    let (ferris_image, ferris_dims) = {
        use std::fs::File;
        let decoder =
            png::Decoder::new(File::open("./examples/textures/ferris_plush.png").unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        (buf, DimsBox::new2(info.width, info.height))
    };
    println!("texture loaded");
    let ferris_texture: Texture<D2, SRgb> = Texture::with_images(
        ferris_dims,
        Some(SRgb::slice_from_raw(&ferris_image)),
        state.clone(),
    )
    .unwrap();
    println!("texture uploaded");

    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

    let mut render_state = RenderState {
        srgb: true,
        viewport: OffsetBox {
            origin: Point2::new(0, 0),
            dims: Vector2::new(512, 512),
        },
        ..RenderState::default()
    };

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let physical_size = logical_size.to_physical(window.get_hidpi_factor());

                    let uniform = Uniforms {
                        tex: ferris_texture.as_dyn(),
                    };
                    render_state.viewport = OffsetBox::new2(
                        0,
                        0,
                        physical_size.width as u32,
                        physical_size.height as u32,
                    );
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(
                        DrawMode::Triangles,
                        ..,
                        &vao,
                        &program,
                        uniform,
                        render_state,
                    );

                    window.swap_buffers().unwrap();
                }
                WindowEvent::CloseRequested => return ControlFlow::Break,
                _ => (),
            },
            _ => (),
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
