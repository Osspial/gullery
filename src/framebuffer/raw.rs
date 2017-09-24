use gl::{self, Gl};
use gl::types::*;

use glsl::{TypeGroup, Scalar};
use buffers::Index;
use vao::BoundVAO;
use uniforms::Uniforms;
use program::BoundProgram;
use colors::Rgba;

use std::mem;
use std::cell::Cell;
use std::marker::PhantomData;
use std::collections::range::RangeArgument;

pub unsafe trait RawFramebuffer {
    fn handle(&self) -> GLuint;
}

pub struct RawDefaultFramebuffer;
unsafe impl RawFramebuffer for RawDefaultFramebuffer {
    #[inline]
    fn handle(&self) -> GLuint {0}
}

// pub struct RawFramebufferTargetRead {
//     bound_fb: Cell<GLuint>
// }

pub struct RawFramebufferTargetDraw {
    bound_fb: Cell<GLuint>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DrawMode {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    LineStripadjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY
    // We don't support patches because that's an OpenGL 4 feature.
    // Patches
}

// pub struct RawBoundFramebufferRead<'a, F>
//     where F: 'a + RawFramebuffer
// {
//     _fb: PhantomData<&'a F>,
//     gl: &'a Gl
// }

pub struct RawBoundFramebufferDraw<'a, F>
    where F: 'a + RawFramebuffer
{
    _fb: PhantomData<&'a mut F>,
    gl: &'a Gl
}

// impl RawFramebufferTargetRead {
//     #[inline]
//     pub fn new() -> RawFramebufferTargetRead {
//         RawFramebufferTargetRead {
//             bound_fb: Cell::new(0)
//         }
//     }

//     #[inline]
//     pub unsafe fn bind<'a, F>(&'a self, framebuffer: &'a F, gl: &'a Gl) -> RawBoundFramebufferRead<'a, F>
//         where F: RawFramebuffer
//     {
//         if self.bound_fb.get() != framebuffer.handle() {
//             self.bound_fb.set(framebuffer.handle());
//             gl.BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer.handle());
//         }

//         RawBoundFramebufferRead {
//             _fb: PhantomData,
//             gl
//         }
//     }

//     #[inline]
//     unsafe fn reset_bind(&self, gl: &Gl) {
//         self.bound_fb.set(0);
//         gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
//     }
// }

impl RawFramebufferTargetDraw {
    #[inline]
    pub fn new() -> RawFramebufferTargetDraw {
        RawFramebufferTargetDraw {
            bound_fb: Cell::new(0)
        }
    }

    #[inline]
    pub unsafe fn bind<'a, F>(&'a self, framebuffer: &'a mut F, gl: &'a Gl) -> RawBoundFramebufferDraw<'a, F>
        where F: RawFramebuffer
    {
        if self.bound_fb.get() != framebuffer.handle() {
            self.bound_fb.set(framebuffer.handle());
            gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer.handle());
        }

        RawBoundFramebufferDraw {
            _fb: PhantomData,
            gl
        }
    }

    // #[inline]
    // unsafe fn reset_bind(&self, gl: &Gl) {
    //     self.bound_fb.set(0);
    //     gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    // }
}

// impl<'a, F> RawBoundFramebufferRead<'a, F>
//     where F: RawFramebuffer {}

impl<'a, F> RawBoundFramebufferDraw<'a, F>
    where F: RawFramebuffer
{
    #[inline]
    pub(crate) fn clear_color(&mut self, color: Rgba<f32>) {
        unsafe{ self.gl.ClearBufferfv(gl::COLOR, 0, &color.r) }
    }

    #[inline]
    pub(crate) fn draw<R, V, I, U>(&mut self, mode: DrawMode, range: R, bound_vao: &BoundVAO<V, I>, _bound_program: &BoundProgram<V, U>)
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms
    {
        let index_type_option = match mem::size_of::<I>() {
            0 => None,
            1 => Some(u8::gl_enum()),
            2 => Some(u16::gl_enum()),
            4 => Some(u32::gl_enum()),
            _ => panic!("Unexpected index size")
        };

        let read_offset = ::bound_to_num_start(range.start(), 0);

        if let Some(index_type) = index_type_option {
            let read_end = ::bound_to_num_end(range.end(), bound_vao.vao().index_buffer().size());
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= GLsizei::max_value() as usize);

            unsafe {
                self.gl.DrawElements(
                    mode.to_gl_enum(),
                    (read_end - read_offset) as GLsizei,
                    index_type,
                    (read_offset * mem::size_of::<I>()) as *const GLvoid
                );
            }
        } else {
            let read_end = ::bound_to_num_end(range.end(), bound_vao.vao().vertex_buffer().size());
            assert!(read_offset <= GLint::max_value() as usize);
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= isize::max_value() as usize);

            unsafe {
                self.gl.DrawArrays(
                    mode.to_gl_enum(),
                    read_offset as GLint,
                    (read_end - read_offset) as GLsizei
                );
            }
        }
    }
}

impl DrawMode {
    #[inline]
    fn to_gl_enum(self) -> GLenum {
        unsafe{ mem::transmute(self) }
    }
}
