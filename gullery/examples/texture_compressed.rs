extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate glutin;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    geometry::{GLVec2, Normalized},
    image_format::{compressed::RGTC, *},
    program::*,
    texture::*,
    vertex::VertexArrayObject,
    ContextState,
};

use glutin::{
    dpi::LogicalSize, ContextBuilder, ControlFlow, Event, EventsLoop, GlContext, GlRequest,
    GlWindow, WindowBuilder, WindowEvent,
};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec2<f32>,
    tex_coord: GLVec2<u16, Normalized>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, RGTC<Rg>>,
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
                pos: GLVec2::new(-1.0, -1.0),
                tex_coord: GLVec2::new(0, !0),
            },
            Vertex {
                pos: GLVec2::new(-1.0, 1.0),
                tex_coord: GLVec2::new(0, 0),
            },
            Vertex {
                pos: GLVec2::new(1.0, 1.0),
                tex_coord: GLVec2::new(!0, 0),
            },
            Vertex {
                pos: GLVec2::new(1.0, -1.0),
                tex_coord: GLVec2::new(!0, !0),
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
    let (image, dims) = {
        use std::{fs::File, io::BufReader};

        let mut file = BufReader::new(File::open("./examples/textures/rg.dds").unwrap());
        let dds = ddsfile::Dds::read(&mut file).unwrap();
        let buf_len = dds.header.linear_size.unwrap() as usize;
        assert_eq!(
            Some(ddsfile::FourCC(ddsfile::FourCC::ATI2)),
            dds.header.spf.fourcc
        );
        (
            dds.data[..buf_len].to_vec(),
            GLVec2::new(dds.header.width, dds.header.height),
        )
    };
    println!("texture loaded");
    let texture =
        Texture::with_images(dims, Some(RGTC::from_raw_slice(&image)), state.clone()).unwrap();
    println!("texture uploaded");

    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

    let mut render_state = RenderState {
        srgb: true,
        viewport: GLVec2::new(0, 0)..=GLVec2::new(512, 512),
        ..RenderState::default()
    };

    let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let physical_size = logical_size.to_physical(window.get_hidpi_factor());

                    let uniform = Uniforms { tex: &texture };
                    render_state.viewport = GLVec2::new(0, 0)
                        ..=GLVec2::new(physical_size.width as u32, physical_size.height as u32);
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
                    default_framebuffer.draw(
                        DrawMode::Triangles,
                        ..,
                        &vao,
                        &program,
                        uniform,
                        &render_state,
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
