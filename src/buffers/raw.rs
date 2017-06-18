use gl;
use gl::types::*;

use std::{ptr, mem};
use std::marker::PhantomData;

pub struct RawBuffer {
    handle: GLuint,
    size: GLsizeiptr
}

pub struct RawBoundBuffer<'a, T: RawBindTarget>{
    buf_size: GLsizeiptr,
    _marker: PhantomData<&'a mut T>
}

pub trait RawBindTarget: 'static + Sized {
    fn target() -> GLenum;

    #[inline]
    fn bind<'a>(&'a mut self, buffer: &RawBuffer) -> RawBoundBuffer<'a, Self> {
        unsafe{ gl::BindBuffer(Self::target(), buffer.handle) }
        RawBoundBuffer {
            buf_size: buffer.size,
            _marker: PhantomData
        }
    }
    #[inline]
    fn reset_bind(&mut self) {
        unsafe{ gl::BindBuffer(Self::target(), 0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BufferUsage {
    StreamDraw = gl::STREAM_DRAW,
    StreamRead = gl::STREAM_READ,
    StreamCopy = gl::STREAM_COPY,
    StaticDraw = gl::STATIC_DRAW,
    StaticRead = gl::STATIC_READ,
    StaticCopy = gl::STATIC_COPY,
    DynamicDraw = gl::DYNAMIC_DRAW,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY
}

pub mod targets {
    use super::*;
    macro_rules! bind_target {
        ($(
            pub target $target_name:ident = $target_enum:expr;
        )*) => ($(
            pub struct $target_name;
            impl RawBindTarget for $target_name {
                #[inline]
                fn target() -> GLenum {$target_enum}
            }
        )*);
    }

    bind_target!{
        pub target Array = gl::ARRAY_BUFFER;
        pub target CopyRead = gl::COPY_READ_BUFFER;
        pub target CopyWrite = gl::COPY_WRITE_BUFFER;
        pub target DrawIndirect = gl::DRAW_INDIRECT_BUFFER;
        pub target ElementArray = gl::ELEMENT_ARRAY_BUFFER;
        pub target PixelPack = gl::PIXEL_PACK_BUFFER;
        pub target PixelUnpack = gl::PIXEL_UNPACK_BUFFER;
        pub target Texture = gl::TEXTURE_BUFFER;
        pub target TransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER;
        pub target Uniform = gl::UNIFORM_BUFFER;
    }
}



impl RawBuffer {
    #[inline]
    pub fn new() -> RawBuffer {
        unsafe {
            let mut handle = 0;
            gl::GenBuffers(1, &mut handle);

            RawBuffer{ 
                handle,
                size: 0
            }
        }
    }

    /// Get the size of the raw buffer
    #[inline]
    pub fn size(&self) -> GLsizeiptr {
        self.size
    }
}

impl Drop for RawBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

impl<'a, T: RawBindTarget> RawBoundBuffer<'a, T> {
    #[inline]
    pub fn get_data(&self, offset: usize, buf: &mut [u8]) {
        if offset + buf.len() <= self.buf_size as usize {
            unsafe {gl::GetBufferSubData(
                T::target(), 
                offset as GLintptr, 
                buf.len() as GLsizeiptr, 
                buf.as_mut_ptr() as *mut GLvoid
            )};
        } else {
            panic!("Attempted to get data from buffer where offset + request length exceeded buffer length");
        }
    }

    #[inline]
    pub fn sub_data(&self, offset: usize, data: &[u8]) {
        if offset + data.len() <= self.buf_size as usize {
            unsafe {gl::BufferSubData(
                T::target(), 
                offset as GLintptr, 
                data.len() as GLsizeiptr, 
                data.as_ptr() as *const GLvoid
            )};
        } else {
            panic!("Attempted to upload data to buffer where offset + data length exceeded buffer length");
        }
    }

    #[inline]
    pub fn alloc_data(&self, size: usize, usage: BufferUsage) {
        unsafe {gl::BufferData(
            T::target(),
            size as GLsizeiptr,
            ptr::null_mut(),
            mem::transmute(usage)
        )};
    }

    #[inline]
    pub fn alloc_upload(&self, data: &[u8], usage: BufferUsage) {
        unsafe {gl::BufferData(
            T::target(),
            data.len() as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            mem::transmute(usage)
        )};
    }
}
