use gl;
use gl::types::*;

use std::{ptr, mem};
use std::ops::Deref;
use std::marker::PhantomData;

pub struct RawBuffer<T: Copy> {
    handle: GLuint,
    size: GLsizeiptr,
    _marker: PhantomData<T>
}

// The RawBoundBuffer types are #[repr(C)] because deref coercing from RawBoundBufferMut to 
// RawBoundBuffer requires a pointer type change, and we want to be sure that the memory layouts are
// identical between the two types.
#[repr(C)]
pub struct RawBoundBuffer<'bind, 'buf, T, B>
    where B: 'bind + RawBindTarget,
          T: 'buf + Copy
{
    bind: PhantomData<&'bind B>,
    buffer: &'buf RawBuffer<T>
} 

#[repr(C)]
pub struct RawBoundBufferMut<'bind, 'buf, T, B>
    where B: 'bind + RawBindTarget,
          T: 'buf + Copy
{
    bind: PhantomData<&'bind B>,
    buffer: &'buf mut RawBuffer<T>,
}

pub trait RawBindTarget: 'static + Sized {
    fn target() -> GLenum;

    #[inline]
    fn bind<'buf: 'bind, 'bind, T: Copy>(&'bind mut self, buffer: &'buf RawBuffer<T>) -> RawBoundBuffer<'buf, 'bind, T, Self> {
        unsafe{ gl::BindBuffer(Self::target(), buffer.handle) }
        RawBoundBuffer {
            bind: PhantomData,
            buffer
        }
    }
    #[inline]
    fn bind_mut<'buf: 'bind, 'bind, T: Copy>(&'bind mut self, buffer: &'buf mut RawBuffer<T>) -> RawBoundBufferMut<'buf, 'bind, T, Self> {
        unsafe{ gl::BindBuffer(Self::target(), buffer.handle) }
        RawBoundBufferMut {
            bind: PhantomData,
            buffer
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



impl<T: Copy> RawBuffer<T> {
    #[inline]
    pub fn new() -> RawBuffer<T> {
        unsafe {
            let mut handle = 0;
            gl::GenBuffers(1, &mut handle);

            RawBuffer { 
                handle,
                size: 0,
                _marker: PhantomData
            }
        }
    }

    /// Get the size of the raw buffer
    #[inline]
    pub fn size(&self) -> usize {
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

impl<'bind, 'buf, T, B> RawBoundBuffer<'bind, 'buf, T, B>
    where B: 'bind + RawBindTarget,
          T: 'buf + Copy
{
    #[inline]
    pub fn get_data(&self, offset: usize, buf: &mut [T]) {
        if offset + buf.len() <= self.buffer.size as usize {
            unsafe {gl::GetBufferSubData(
                B::target(), 
                offset as GLintptr, 
                (buf.len() * mem::size_of::<T>()) as GLsizeiptr, 
                buf.as_mut_ptr() as *mut GLvoid
            )};
        } else {
            panic!("Attempted to get data from buffer where offset + request length exceeded buffer length");
        }
    }

    #[inline]
    pub fn buffer(&self) -> &RawBuffer<T> {
        &self.buffer
    }
}

impl<'bind, 'buf, T, B> RawBoundBufferMut<'bind, 'buf, T, B>
    where B: 'bind + RawBindTarget,
          T: 'buf + Copy
{
    #[inline]
    pub fn sub_data(&mut self, offset: usize, data: &[T]) {
        if offset + data.len() <= self.buffer.size as usize {
            unsafe {gl::BufferSubData(
                B::target(), 
                offset as GLintptr, 
                (data.len() * mem::size_of::<T>()) as GLsizeiptr, 
                data.as_ptr() as *const GLvoid
            )};
        } else {
            panic!("Attempted to upload data to buffer where offset + data length exceeded buffer length");
        }
    }

    #[inline]
    pub fn alloc_data(&mut self, size: usize, usage: BufferUsage) {
        assert!(size < isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::target(),
            (size * mem::size_of::<T>()) as GLsizeiptr,
            ptr::null_mut(),
            mem::transmute(usage)
        )};

        self.buffer.size = size as GLsizeiptr;
    }

    #[inline]
    pub fn alloc_upload(&mut self, data: &[T], usage: BufferUsage) {
        assert!(data.len() < isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::target(),
            (data.len() * mem::size_of::<T>()) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            mem::transmute(usage)
        )};

        self.buffer.size = data.len() as GLsizeiptr;
    }
}

impl<'bind, 'buf, T, B> Deref for RawBoundBufferMut<'bind, 'buf, T, B>
    where B: 'bind + RawBindTarget,
          T: 'buf + Copy
{
    type Target = RawBoundBuffer<'bind, 'buf, T, B>;

    fn deref(&self) -> &RawBoundBuffer<'bind, 'buf, T, B> {
        unsafe{ &*(self as *const _ as *const RawBoundBuffer<'bind, 'buf, T, B>) }
    }
}
