#![feature(never_type)]

extern crate gullery;
#[macro_use]
extern crate gullery_macros;
extern crate glutin;
extern crate png;

use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    geometry::{GLVec2, NonNormalized},
    image_format::*,
    program::*,
    texture::*,
    vertex::VertexArrayObject,
    ContextState,
};

use glutin::*;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec2<f32>,
    color: Rgb<u8>,
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms {
    offset: GLVec2<u32, NonNormalized>,
}

#[derive(Attachments)]
struct Attachments<'a> {
    color: &'a mut Texture<D2, SRgb>,
    color_inverted: Texture<D2, SRgb>,
}

fn main() {
    let (size_x, size_y) = (512, 512);
    let el = EventsLoop::new();
    let headless = Context::new(
        &el,
        ContextBuilder::new()
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))),
        false,
    )
    .unwrap();
    unsafe { headless.make_current().unwrap() };
    let state = unsafe { ContextState::new(|addr| headless.get_proc_address(addr)) };

    let vertex_buffer = Buffer::with_data(
        BufferUsage::StaticDraw,
        &[
            Vertex {
                pos: GLVec2::new(-1.0, -1.0),
                color: Rgb::new(255, 0, 0),
            },
            Vertex {
                pos: GLVec2::new(0.0, 1.0),
                color: Rgb::new(0, 255, 0),
            },
            Vertex {
                pos: GLVec2::new(1.0, -1.0),
                color: Rgb::new(0, 0, 255),
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

    let mut texture =
        Texture::with_mip_count(GLVec2::new(size_x, size_y), 1, state.clone()).unwrap();
    let mut fbo_attached = FramebufferObjectAttached {
        fbo: FramebufferObject::new(state.clone()),
        attachments: Attachments {
            color: &mut texture,
            color_inverted: Texture::with_mip_count(GLVec2::new(size_x, size_y), 1, state.clone())
                .unwrap(),
        },
    };

    let uniform = TriUniforms {
        offset: GLVec2::new(0, 0),
    };
    let render_state = RenderState {
        srgb: true,
        viewport: GLVec2::new(0, 0)..=GLVec2::new(size_x, size_y),
        ..RenderState::default()
    };
    fbo_attached.clear_depth(1.0);
    fbo_attached.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
    fbo_attached.draw(
        DrawMode::Triangles,
        ..,
        &vao,
        &program,
        &uniform,
        &render_state,
    );

    let (width, height) = (size_x, size_y);
    let mut data_buffer = vec![SRgb::new(0, 0, 0); (width * height) as usize * 2];
    fbo_attached.read_pixels_attachment(
        GLVec2::new(0, 0)..=GLVec2::new(width, height),
        &mut data_buffer[(width * height) as usize..],
        |a| &a.color,
    );
    fbo_attached.read_pixels_attachment(
        GLVec2::new(0, 0)..=GLVec2::new(width, height),
        &mut data_buffer[0..(width * height) as usize],
        |a| &a.color_inverted,
    );

    // OpenGL outputs the pixels with a top-left origin, but PNG exports then with a bottom-right
    // origin. This accounts for that.
    {
        let mut lines_mut = data_buffer.chunks_mut(width as usize);
        while let (Some(top), Some(bottom)) = (lines_mut.next(), lines_mut.next_back()) {
            for (t, b) in top.iter_mut().zip(bottom.iter_mut()) {
                ::std::mem::swap(t, b);
            }
        }
    }

    use png::HasParameters;
    use std::{fs::File, io::BufWriter};
    let file = File::create("target/output_pixels.png").unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height * 2);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(SRgb::to_raw_slice(&data_buffer))
        .unwrap();
}

const VERTEX_SHADER: &str = r#"
    #version 330

    in vec2 pos;
    in vec3 color;
    out vec3 vert_color;

    uniform uvec2 offset;

    void main() {
        vert_color = color;
        gl_Position = vec4(pos + vec2(offset), 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec3 vert_color;
    out vec4 color;
    out vec4 color_inverted;

    void main() {
        color = vec4(vert_color, 1.0);
        color_inverted = vec4(1.0 - vert_color, 1.0);
    }
"#;
