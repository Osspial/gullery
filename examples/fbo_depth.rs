#![feature(never_type)]

extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate cgmath_geometry;
extern crate glutin;
extern crate png;

extern crate num_traits;

use gullery::ContextState;
use gullery::buffer::*;
use gullery::texture::{*, targets::*, sample_parameters::Swizzle};
use gullery::framebuffer::{*, render_state::*};
use gullery::program::*;
use gullery::image_format::*;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::{cgmath, D2};
use cgmath_geometry::rect::{DimsBox, OffsetBox};

use cgmath::*;

use glutin::*;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector3<f32>
}

#[derive(Vertex, Clone, Copy)]
struct TextureVertex {
    pos: Vector2<f32>,
    uv: Vector2<u16>
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    offset: Vector2<f32>,
    tex: &'a Texture<SimpleTex<Depth16, D2>>
}

#[derive(Attachments)]
struct Attachments<'a> {
    color: &'a mut Texture<SimpleTex<SRgb, D2>>,
    depth: &'a mut Texture<SimpleTex<Depth16, D2>>
}

fn main() {
    let (size_x, size_y) = (512, 512);

    let mut events_loop = EventsLoop::new();
    let window = GlWindow::new(
        WindowBuilder::new().with_dimensions(size_x * 2, size_y),
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

    let mut color_texture = Texture::new(DimsBox::new2(size_x, size_y), 1, state.clone()).unwrap();
    let mut depth_texture = Texture::new(DimsBox::new2(size_x, size_y), 1, state.clone()).unwrap();

    {
        let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
            Vertex {
                pos: Vector3::new(-1.0, -1.0, -1.0),
            },
            Vertex {
                pos: Vector3::new( 0.0,  1.0, 1.0),
            },
            Vertex {
                pos: Vector3::new( 1.0,  -1.0, -1.0),
            },
        ], state.clone());
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
                depth: &mut depth_texture
            }
        };
        let render_state = RenderState {
            srgb: true,
            viewport: OffsetBox::new2(0, 0, size_x, size_y),
            depth_test: Some(DepthStencilFunc::Less),
            ..RenderState::default()
        };
        fbo_attached.clear_depth(1.0);
        fbo_attached.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
        fbo_attached.draw(DrawMode::Triangles, .., &vao, &program, (), render_state);
    }

    {
        let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
            TextureVertex {
                pos: Vector2::new(-1.0, -1.0),
                uv: Vector2::new(0, 0)
            },
            TextureVertex {
                pos: Vector2::new(-1.0,  1.0),
                uv: Vector2::new(0, !0)
            },
            TextureVertex {
                pos: Vector2::new( 0.0,  1.0),
                uv: Vector2::new(!0, !0)
            },
            TextureVertex {
                pos: Vector2::new( 0.0, -1.0),
                uv: Vector2::new(!0, 0)
            },
        ], state.clone());
        let index_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
            0, 1, 2,
            2, 3, 0u16
        ], state.clone());
        let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));

        let vertex_shader = Shader::new(TEX_TO_WINDOW_VERTEX_SHADER, state.clone()).unwrap();
        let fragment_shader = Shader::new(TEX_TO_WINDOW_FRAGMENT_DEPTH_SHADER, state.clone()).unwrap();
        let (program, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();

        let mut render_state = RenderState {
            srgb: true,
            viewport: OffsetBox {
                origin: Point2::new(0, 0),
                dims: Vector2::new(512, 512)
            },
            ..RenderState::default()
        };

        depth_texture.swizzle_mask(Rgba::new(Swizzle::Red, Swizzle::Red, Swizzle::Red, Swizzle::One));

        let mut default_framebuffer = DefaultFramebuffer::new(state.clone());
        events_loop.run_forever(|event| {
            match event {
                Event::WindowEvent{event, ..} => match event {
                    WindowEvent::Resized(size_x, size_y) => {
                        window.context().resize(size_x, size_y);
                        let uniform = Uniforms {
                            offset: Vector2::new(0.0, 0.0),
                            tex: &depth_texture
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
    out vec2 tex_coord;

    void main() {
        gl_Position = vec4(pos, 0, 1);
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
