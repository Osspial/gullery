#![feature(never_type)]

extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate glutin;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    geometry::{GLSLFloat, GLVec2, GLVec3, Normalized},
    image_format::*,
    program::*,
    texture::{sample_parameters::Swizzle, *},
    vertex::VertexArrayObject,
    ContextState,
};

use glutin::{dpi::LogicalSize, *};

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec3<f32>,
}

#[derive(Vertex, Clone, Copy)]
struct TextureVertex {
    pos: GLVec2<f32>,
    uv: GLVec2<u16, Normalized>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    offset: GLVec2<f32>,
    tex: &'a Texture<D2, dyn ImageFormat<ScalarType = GLSLFloat>>,
}

#[derive(Attachments)]
struct Attachments<'a> {
    color: &'a mut Texture<D2, SRgb>,
    depth: &'a mut Texture<D2, Depth16>,
}

fn main() {
    let (size_x, size_y) = (512, 512);

    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(LogicalSize::new(size_x as f64 * 2.0, size_y as f64)),
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

    let mut color_texture =
        Texture::with_mip_count(GLVec2::new(size_x, size_y), 1, state.clone()).unwrap();
    let mut depth_texture =
        Texture::with_mip_count(GLVec2::new(size_x, size_y), 1, state.clone()).unwrap();

    {
        let vertex_buffer = Buffer::with_data(
            BufferUsage::StaticDraw,
            &[
                Vertex {
                    pos: GLVec3::new(-1.0, -1.0, -1.0),
                },
                Vertex {
                    pos: GLVec3::new(0.0, 1.0, 1.0),
                },
                Vertex {
                    pos: GLVec3::new(1.0, -1.0, -1.0),
                },
            ],
            state.clone(),
        );
        let vao = VertexArrayObject::<_, !>::new(vertex_buffer, None);

        let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
        let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
        let (program, warnings) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();
        for w in warnings {
            println!("Warning: {}", w);
        }

        let mut fbo_attached = FramebufferObjectAttached {
            fbo: FramebufferObject::new(state.clone()),
            attachments: Attachments {
                color: &mut color_texture,
                depth: &mut depth_texture,
            },
        };
        let render_state = RenderState {
            srgb: true,
            viewport: GLVec2::new(0, 0)..=GLVec2::new(size_x, size_y),
            depth_test: Some(DepthStencilFunc::Less),
            ..RenderState::default()
        };
        fbo_attached.clear_depth(1.0);
        fbo_attached.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
        fbo_attached.draw(DrawMode::Triangles, .., &vao, &program, &(), &render_state);
    }

    {
        let vertex_buffer = Buffer::with_data(
            BufferUsage::StaticDraw,
            &[
                TextureVertex {
                    pos: GLVec2::new(-1.0, -1.0),
                    uv: GLVec2::new(0, 0),
                },
                TextureVertex {
                    pos: GLVec2::new(-1.0, 1.0),
                    uv: GLVec2::new(0, !0),
                },
                TextureVertex {
                    pos: GLVec2::new(0.0, 1.0),
                    uv: GLVec2::new(!0, !0),
                },
                TextureVertex {
                    pos: GLVec2::new(0.0, -1.0),
                    uv: GLVec2::new(!0, 0),
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

        let vertex_shader = Shader::new(TEX_TO_WINDOW_VERTEX_SHADER, state.clone()).unwrap();
        let fragment_shader =
            Shader::new(TEX_TO_WINDOW_FRAGMENT_DEPTH_SHADER, state.clone()).unwrap();
        let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

        let mut render_state = RenderState {
            srgb: true,
            viewport: GLVec2::new(0, 0)..=GLVec2::new(512, 512),
            ..RenderState::default()
        };

        depth_texture.swizzle_read(Swizzle::Red, Swizzle::Red, Swizzle::Red, Swizzle::One);

        let mut default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();
        events_loop.run_forever(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(logical_size) => {
                        let physical_size = logical_size.to_physical(window.get_hidpi_factor());
                        window.resize(physical_size);
                        let uniform = Uniforms {
                            offset: GLVec2::new(0.0, 0.0),
                            tex: color_texture.as_dyn(),
                        };
                        render_state.viewport = GLVec2::new(0, 0)
                            ..=GLVec2::new(physical_size.width as _, physical_size.height as _);
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

                        let uniform = Uniforms {
                            offset: GLVec2::new(1.0, 0.0),
                            tex: depth_texture.as_dyn(),
                        };
                        default_framebuffer.draw(
                            DrawMode::Triangles,
                            ..,
                            &vao,
                            &program,
                            &uniform,
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
}

const VERTEX_SHADER: &str = r#"
    #version 330

    in vec3 pos;

    void main() {
        gl_Position = vec4(pos, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    out vec4 color;

    void main() {
        color = vec4(1, 0, 0, 1);
    }
"#;

const TEX_TO_WINDOW_VERTEX_SHADER: &str = r#"
    #version 330

    in vec2 pos;
    in vec2 uv;
    uniform vec2 offset;

    out vec2 tex_coord;

    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        tex_coord = uv;
    }
"#;

const TEX_TO_WINDOW_FRAGMENT_DEPTH_SHADER: &str = r#"
    #version 330

    in vec2 tex_coord;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = texture(tex, tex_coord);
    }
"#;
