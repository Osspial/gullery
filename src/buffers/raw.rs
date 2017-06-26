use gl;
use gl::types::*;

use std::{ptr, mem};
use std::ops::Deref;
use std::cell::Cell;
use std::marker::PhantomData;
use std::error::Error;
use std::fmt::{self, Display};

pub struct RawBuffer<T: Copy> {
    size: GLsizeiptr,
    handle: GLuint,
    _marker: PhantomData<(T, *const ())>
}

// The RawBoundBuffer types are #[repr(C)] because deref coercing from RawBoundBufferMut to
// RawBoundBuffer requires a pointer type change, and we want to be sure that the memory layouts are
// identical between the two types.
#[repr(C)]
pub struct RawBoundBuffer<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    bind: PhantomData<&'a B>,
    buffer: &'a RawBuffer<T>
}

#[repr(C)]
pub struct RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    bind: PhantomData<&'a B>,
    buffer: &'a mut RawBuffer<T>,
}

const USAGE_OFFSET: GLenum = 35039;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BufferUsage {
    StreamDraw = (gl::STREAM_DRAW - USAGE_OFFSET) as u8,
    StreamRead = (gl::STREAM_READ - USAGE_OFFSET) as u8,
    StreamCopy = (gl::STREAM_COPY - USAGE_OFFSET) as u8,
    StaticDraw = (gl::STATIC_DRAW - USAGE_OFFSET) as u8,
    StaticRead = (gl::STATIC_READ - USAGE_OFFSET) as u8,
    StaticCopy = (gl::STATIC_COPY - USAGE_OFFSET) as u8,
    DynamicDraw = (gl::DYNAMIC_DRAW - USAGE_OFFSET) as u8,
    DynamicRead = (gl::DYNAMIC_READ - USAGE_OFFSET) as u8,
    DynamicCopy = (gl::DYNAMIC_COPY - USAGE_OFFSET) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocError {
    OutOfMemory
}

pub unsafe trait RawBindTarget: 'static + Sized {
    const TARGET: GLenum;

    fn bound_buffer(&self) -> &Cell<GLuint>;

    #[inline]
    unsafe fn bind<'a, T: Copy>(&'a self, buffer: &'a RawBuffer<T>) -> RawBoundBuffer<'a, T, Self> {
        let handle = buffer.handle;
        let bound_buffer = self.bound_buffer();
        if bound_buffer.get() != handle {
            gl::BindBuffer(Self::TARGET, buffer.handle);
            bound_buffer.set(handle);
        }

        RawBoundBuffer {
            bind: PhantomData,
            buffer
        }
    }
    #[inline]
    unsafe fn bind_mut<'a, T: Copy>(&'a self, buffer: &'a mut RawBuffer<T>) -> RawBoundBufferMut<'a, T, Self> {
        let handle = buffer.handle;
        let bound_buffer = self.bound_buffer();
        if bound_buffer.get() != handle {
            gl::BindBuffer(Self::TARGET, buffer.handle);
            bound_buffer.set(handle);
        }
        RawBoundBufferMut {
            bind: PhantomData,
            buffer
        }
    }
    #[inline]
    unsafe fn reset_bind(&mut self) {
        self.bound_buffer().set(0);
        gl::BindBuffer(Self::TARGET, 0)
    }
}

pub mod targets {
    use super::*;
    macro_rules! raw_bind_target {
        ($(
            pub target $target_name:ident = $target_enum:expr;
        )*) => ($(
            pub struct $target_name {
                bound_buffer: Cell<GLuint>,
                _marker: PhantomData<*const ()>
            }
            impl $target_name {
                #[inline]
                pub(crate) unsafe fn new() -> $target_name {
                    $target_name {
                        bound_buffer: Cell::new(0),
                        _marker: PhantomData
                    }
                }
            }
            unsafe impl RawBindTarget for $target_name {
                const TARGET: GLenum = $target_enum;

                #[inline]
                fn bound_buffer(&self) -> &Cell<GLuint> {
                    &self.bound_buffer
                }
            }
        )*);
    }

    raw_bind_target!{
        // pub target RawArray = gl::ARRAY_BUFFER;
        pub target RawCopyRead = gl::COPY_READ_BUFFER;
        pub target RawCopyWrite = gl::COPY_WRITE_BUFFER;
        // pub target RawDrawIndirect = gl::DRAW_INDIRECT_BUFFER;
        // pub target RawElementArray = gl::ELEMENT_ARRAY_BUFFER;
        // pub target RawPixelPack = gl::PIXEL_PACK_BUFFER;
        // pub target RawPixelUnpack = gl::PIXEL_UNPACK_BUFFER;
        // pub target RawTexture = gl::TEXTURE_BUFFER;
        // pub target RawTransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER;
        // pub target RawUniform = gl::UNIFORM_BUFFER;
    }
}


impl<T: Copy> RawBuffer<T> {
    #[inline]
    pub(crate) fn new() -> RawBuffer<T> {
        unsafe {
            let mut handle = 0;
            gl::GenBuffers(1, &mut handle);

            RawBuffer {
                size: 0,
                handle,
                _marker: PhantomData
            }
        }
    }

    /// Get the size of the raw buffer
    #[inline]
    pub(crate) fn size(&self) -> usize {
        self.size as usize
    }
}

impl<T: Copy> Drop for RawBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

impl<'a, T, B> RawBoundBuffer<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub(crate) fn get_data(&self, offset: usize, buf: &mut [T]) {
        if offset + buf.len() <= self.buffer.size as usize {
            unsafe {gl::GetBufferSubData(
                B::TARGET,
                offset as GLintptr,
                (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                buf.as_mut_ptr() as *mut GLvoid
            )};
        } else {
            panic!("Attempted to get data from buffer where offset + request length exceeded buffer length");
        }
    }
}

impl<'a, T, B> RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub(crate) fn sub_data(&mut self, offset: usize, data: &[T]) {
        if offset + data.len() <= self.buffer.size as usize {
            unsafe {gl::BufferSubData(
                B::TARGET,
                offset as GLintptr,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid
            )};
        } else {
            panic!("Attempted to upload data to buffer where offset + data length exceeded buffer length");
        }
    }

    #[inline]
    pub(crate) fn alloc_size(&mut self, size: usize, usage: BufferUsage) -> Result<(), AllocError> {
        assert!(size <= isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::TARGET,
            (size * mem::size_of::<T>()) as GLsizeiptr,
            ptr::null_mut(),
            usage.to_gl_enum()
        )};

        let error = unsafe{ gl::GetError() };
        if error == 0 {
            self.buffer.size = size as GLsizeiptr;
            Ok(())
        } else {
            assert_eq!(error, gl::OUT_OF_MEMORY);
            self.buffer.size = 0;
            Err(AllocError::OutOfMemory)
        }
    }

    #[inline]
    pub(crate) fn alloc_upload(&mut self, data: &[T], usage: BufferUsage) -> Result<(), AllocError> {
        assert!(data.len() <= isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::TARGET,
            (data.len() * mem::size_of::<T>()) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            usage.to_gl_enum()
        )};

        let error = unsafe{ gl::GetError() };
        if error == 0 {
            self.buffer.size = data.len() as GLsizeiptr;
            Ok(())
        } else {
            assert_eq!(error, gl::OUT_OF_MEMORY);
            self.buffer.size = 0;
            Err(AllocError::OutOfMemory)
        }
    }
}

impl<'a, T, B> Deref for RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    type Target = RawBoundBuffer<'a, T, B>;

    fn deref(&self) -> &RawBoundBuffer<'a, T, B> {
        unsafe{ &*(self as *const _ as *const RawBoundBuffer<'a, T, B>) }
    }
}

impl BufferUsage {
    #[inline]
    fn to_gl_enum(self) -> GLenum {
        let discriminant: u8 = unsafe{ mem::transmute(self) };
        discriminant as GLenum + USAGE_OFFSET
    }
}

impl Display for AllocError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl Error for AllocError {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            AllocError::OutOfMemory => "Not enough VRAM for allocation"
        }
    }
}
