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

use {ContextState, GLObject, Handle};

use gl::{self, Gl};
use gl::types::*;

use std::{ptr, mem};
use std::ops::Deref;
use RangeArgument;
use std::cell::Cell;
use std::marker::PhantomData;

pub struct RawBuffer<T: Copy> {
    handle: Handle,
    size: usize,
    /// `*const ()` used to opt out of `Send` and `Sync` without relying on the unstable opt-out
    /// features.
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
    buffer: &'a RawBuffer<T>,
    gl: &'a Gl
}

#[repr(C)]
pub struct RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    bind: PhantomData<&'a B>,
    buffer: &'a mut RawBuffer<T>,
    gl: &'a Gl
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

pub unsafe trait RawBindTarget: 'static + Sized {
    const TARGET: GLenum;
    fn bound_buffer(&self) -> &Cell<Option<Handle>>;

    #[inline]
    unsafe fn bind<'a, T: Copy>(&'a self, buffer: &'a RawBuffer<T>, gl: &'a Gl) -> RawBoundBuffer<'a, T, Self> {
        let handle = buffer.handle;
        let bound_buffer = self.bound_buffer();
        if bound_buffer.get() != Some(handle) {
            gl.BindBuffer(Self::TARGET, handle.get());
            bound_buffer.set(Some(handle));
        }

        debug_assert_eq!(
            Some(handle),
            {
                let mut bound = 0;
                gl.GetIntegerv(Self::TARGET, &mut bound);
                Handle::new(bound as u32)
            }
        );
        RawBoundBuffer {
            bind: PhantomData,
            buffer, gl
        }
    }
    #[inline]
    unsafe fn bind_mut<'a, T: Copy>(&'a self, buffer: &'a mut RawBuffer<T>, gl: &'a Gl) -> RawBoundBufferMut<'a, T, Self> {
        self.bind(buffer, gl);
        RawBoundBufferMut {
            bind: PhantomData,
            buffer, gl
        }
    }
    #[inline]
    unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_buffer().set(None);
        gl.BindBuffer(Self::TARGET, 0)
    }
}

pub mod targets {
    use super::*;
    macro_rules! raw_bind_target {
        ($(
            pub target $target_name:ident = $target_enum:expr;
        )*) => ($(
            pub struct $target_name {
                bound_buffer: Cell<Option<Handle>>,
                _marker: PhantomData<*const ()>
            }
            impl $target_name {
                #[inline]
                pub(crate) fn new() -> $target_name {
                    $target_name {
                        bound_buffer: Cell::new(None),
                        _marker: PhantomData
                    }
                }
            }
            unsafe impl RawBindTarget for $target_name {
                const TARGET: GLenum = $target_enum;

                #[inline]
                fn bound_buffer(&self) -> &Cell<Option<Handle>> {
                    &self.bound_buffer
                }
            }
        )*);
    }

    // The ARRAY_BUFFER and ELEMENT_ARRAY_BUFFER targets are implemented in the vertex::vao module,
    // under the VertexArrayObjTarget struct.
    raw_bind_target!{
        pub target RawCopyRead = gl::COPY_READ_BUFFER;
        pub target RawCopyWrite = gl::COPY_WRITE_BUFFER;
        // pub target RawDrawIndirect = gl::DRAW_INDIRECT_BUFFER;
        // pub target RawPixelPack = gl::PIXEL_PACK_BUFFER;
        // pub target RawPixelUnpack = gl::PIXEL_UNPACK_BUFFER;
        // pub target RawTexture = gl::TEXTURE_BUFFER;
        // pub target RawTransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER;
        // pub target RawUniform = gl::UNIFORM_BUFFER;
    }
}


impl<T: Copy> RawBuffer<T> {
    /// Allocate a new RawBuffer on the GPU.
    #[inline]
    pub(crate) fn new(gl: &Gl) -> RawBuffer<T> {
        unsafe {
            let mut handle = 0;

            gl.GenBuffers(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawBuffer {
                handle,
                size: 0,
                _marker: PhantomData
            }
        }
    }

    /// Get the size of the raw buffer
    #[inline]
    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn delete(self, state: &ContextState) {
        unsafe {
            if mem::size_of::<T>() != 0 {
                state.buffer_binds.unbind(&self, &state.gl);
                state.gl.DeleteBuffers(1, &self.handle.get());
            }
        }
    }
}

impl<T: Copy> GLObject for RawBuffer<T> {
    #[inline]
    fn handle(&self) -> Handle {
        self.handle
    }
}

impl<'a, T, B> RawBoundBuffer<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub(crate) fn get_data(&self, offset: usize, buf: &mut [T]) {
        if mem::size_of::<T>() != 0 {
            if offset + buf.len() <= self.buffer.size {
                unsafe {self.gl.GetBufferSubData(
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

    #[inline]
    pub(crate) fn copy_to<C, R>(&self, dest_bind: &mut RawBoundBufferMut<T, C>, self_range: R, write_offset: usize)
        where C: RawBindTarget,
              R: RangeArgument<usize>
    {
        if mem::size_of::<T>() != 0 {
            let read_offset = ::bound_to_num_start(self_range.start(), 0);
            let read_end = ::bound_to_num_end(self_range.end(), self.buffer.size);
            assert!(read_end <= isize::max_value() as usize);

            let size = read_offset.checked_sub(read_end)
                .expect(&format!("Copy range starts at {} but ends at {}", read_offset, read_end));

            if read_end > self.buffer.size {
                panic!("Read index {} out of range for buffer of length {}", read_end, self.buffer.size);
            } else if write_offset + size > dest_bind.buffer.size {
                panic!("Write offset {} with read length {} out of range for buffer of length {}", write_offset, size, dest_bind.buffer.size);
            } else if size > 0 {
                unsafe {self.gl.CopyBufferSubData(
                    B::TARGET, C::TARGET,
                    read_offset as GLintptr, write_offset as GLintptr,
                    size as GLsizeiptr
                )}
            }
        }
    }
}

impl<'a, T, B> RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub(crate) fn sub_data(&mut self, offset: usize, data: &[T]) {
        assert!(offset + data.len() <= isize::max_value() as usize);
        if mem::size_of::<T>() != 0 {
            if offset + data.len() <= self.buffer.size {
                unsafe {self.gl.BufferSubData(
                    B::TARGET,
                    offset as GLintptr,
                    (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                    data.as_ptr() as *const GLvoid
                )};
            } else {
                panic!("Attempted to upload data to buffer where offset + data length exceeded buffer length");
            }
        }
    }

    #[inline]
    pub(crate) fn alloc_size(&mut self, size: usize, usage: BufferUsage) {
        assert!(size <= isize::max_value() as usize);
        if mem::size_of::<T>() != 0 {
            unsafe {self.gl.BufferData(
                B::TARGET,
                (size * mem::size_of::<T>()) as GLsizeiptr,
                ptr::null_mut(),
                usage.to_gl_enum()
            )};

            let error = unsafe{ self.gl.GetError() };
            if error == 0 {
                self.buffer.size = size;
            } else {
                if error == gl::OUT_OF_MEMORY {
                    panic!("OpenGL out of memory!");
                } else {
                    panic!("Unexpected OpenGL error: {}", error);
                }
            }
        }
    }

    #[inline]
    pub(crate) fn alloc_upload(&mut self, data: &[T], usage: BufferUsage) {
        assert!(data.len() <= isize::max_value() as usize);
        if mem::size_of::<T>() != 0 {
            unsafe {self.gl.BufferData(
                B::TARGET,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid,
                usage.to_gl_enum()
            )};

            let error = unsafe{ self.gl.GetError() };
            if error == 0 {
                self.buffer.size = data.len();
            } else {
                if error == gl::OUT_OF_MEMORY {
                    panic!("OpenGL out of memory!");
                } else {
                    panic!("Unexpected OpenGL error: {}", error);
                }
            }
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
