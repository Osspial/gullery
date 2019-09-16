extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate glutin;
extern crate png;

mod helper;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    glsl::{GLSLFloat, GLVec2, Normalized},
    image_format::*,
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
    let images = [
        helper::load_png("./textures/mips/512.png").unwrap(),
        helper::load_png("./textures/mips/256.png").unwrap(),
        helper::load_png("./textures/mips/128.png").unwrap(),
        helper::load_png("./textures/mips/64.png").unwrap(),
        helper::load_png("./textures/mips/32.png").unwrap(),
        helper::load_png("./textures/mips/16.png").unwrap(),
        helper::load_png("./textures/mips/8.png").unwrap(),
    ];
    let dims = images[0].1;
    println!("texture loaded");
    let mip_texture: Texture<D2, Rgba> = Texture::with_images(
        dims,
        images.iter().map(|(mip, _)| Rgba::from_raw_slice(&mip[..])),
        state.clone(),
    )
    .unwrap();
    println!("texture uploaded");

    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

    let mut render_state = RenderState {
        srgb: true,
        viewport: GLVec2::new(0, 0)..=GLVec2::new(512, 512),
        blend: BlendFuncs {
            src_rgb: BlendFunc::SrcAlpha,
            dst_rgb: BlendFunc::OneMinusSrcAlpha,
            src_alpha: BlendFunc::One,
            dst_alpha: BlendFunc::Zero,
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
                        tex: mip_texture.as_dyn(),
                    };
                    render_state.viewport = GLVec2::new(0, 0)
                        ..=GLVec2::new(physical_size.width as u32, physical_size.height as u32);
                    default_framebuffer.clear_depth(1.0);
                    default_framebuffer.clear_color_all(Rgba::new(1.0, 1.0, 1.0, 1.0));
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
