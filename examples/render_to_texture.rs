// Copyright 2018 Osspial
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
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
use gullery::texture::{*, targets::*};
use gullery::framebuffer::{*, render_state::*};
use gullery::program::*;
use gullery::color::*;
use gullery::vertex::VertexArrayObject;

use cgmath_geometry::cgmath;
use cgmath_geometry::{DimsBox, OffsetBox};

use cgmath::*;

use glutin::*;

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    color: Rgb<u8>
}

#[derive(Clone, Copy, Uniforms)]
struct TriUniforms {
    offset: Point2<u32>
}

#[derive(Attachments)]
struct Attachments<'a> {
    color: &'a mut Texture<SimpleTex<SRgb, DimsBox<Point2<u32>>>>,
    color_inverted: Texture<SimpleTex<SRgb, DimsBox<Point2<u32>>>>,
}

fn main() {
    let (size_x, size_y) = (512, 512);
    let headless =
        HeadlessRendererBuilder::new(size_x, size_y)
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .build().unwrap();
    unsafe{ headless.make_current().unwrap() };
    let state = unsafe{ ContextState::new(|addr| headless.get_proc_address(addr)) };

    let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &[
        Vertex {
            pos: Vector2::new(-1.0, -1.0),
            color: Rgb::new(255, 0, 0)
        },
        Vertex {
            pos: Vector2::new( 0.0,  1.0),
            color: Rgb::new(0, 255, 0)
        },
        Vertex {
            pos: Vector2::new( 1.0,  -1.0),
            color: Rgb::new(0, 0, 255)
        },
    ], state.clone());
    let vao = VertexArrayObject::<_, !>::new(vertex_buffer, None);


    let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
    let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
    let (program, warnings) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();
    for w in warnings {
        println!("Warning: {}", w);
    }

    let mut texture = Texture::new(DimsBox::new2(size_x, size_y), 1, state.clone()).unwrap();
    let mut fbo_attached = FramebufferObjectAttached {
        fbo: FramebufferObject::new(state.clone()),
        attachments: Attachments {
            color: &mut texture,
            color_inverted: Texture::new(DimsBox::new2(size_x, size_y), 1, state.clone()).unwrap(),
        }
    };

    let uniform = TriUniforms {
        offset: Point2::new(0, 0)
    };
    let render_state = RenderState {
        srgb: true,
        viewport: OffsetBox::new2(0, 0, size_x, size_y),
        ..RenderState::default()
    };
    fbo_attached.clear_depth(1.0);
    fbo_attached.clear_color(Rgba::new(0.0, 0.0, 0.0, 1.0));
    fbo_attached.draw(DrawMode::Triangles, .., &vao, &program, uniform, render_state);

    let (width, height) = (size_x, size_y);
    let mut data_buffer = vec![SRgb::new(0, 0, 0); (width * height) as usize * 2];
    fbo_attached.read_pixels_fbo(
        OffsetBox::new2(0, 0, width, height),
        &mut data_buffer[(width * height) as usize..],
        |a| &a.color
    );
    fbo_attached.read_pixels_fbo(
        OffsetBox::new2(0, 0, width, height),
        &mut data_buffer[0..(width * height) as usize],
        |a| &a.color_inverted
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

    use std::fs::File;
    use std::io::BufWriter;
    use png::HasParameters;
    let file = File::create("target/output_pixels.png").unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height * 2);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(SRgb::to_raw_slice(&data_buffer)).unwrap();
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
