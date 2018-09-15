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

mod raw;
use self::raw::*;

use glsl::TypeGroup;
use buffers::{Index, Buffer, BufferUsage};

use std::{mem, ptr};

pub struct VertexArrayObj<V: TypeGroup, I: Index> {
    raw: RawVAO<V>,
    vertex_buffer: Buffer<V>,
    index_buffer: Buffer<I>
}

pub(crate) struct VAOTarget(RawVAOTarget);
pub(crate) struct BoundVAO<'a, V: TypeGroup, I: Index> {
    vao: &'a VertexArrayObj<V, I>,
    _bind: RawBoundVAO<'a, V>
}

impl<V: TypeGroup, I: Index> VertexArrayObj<V, I> {
    pub fn new(vertex_buffer: Buffer<V>, index_buffer: Buffer<I>) -> VertexArrayObj<V, I> {
        if vertex_buffer.state().as_ref() as *const _ != index_buffer.state().as_ref() as *const _ {
             panic!("vertex buffer and index buffer using different contexts");
        }

        VertexArrayObj {
            raw: RawVAO::new(&vertex_buffer.state().gl),
            vertex_buffer,
            index_buffer
        }
    }

    #[inline]
    pub fn vertex_buffer(&self) -> &Buffer<V> {
        &self.vertex_buffer
    }

    #[inline]
    pub fn vertex_buffer_mut(&mut self) -> &mut Buffer<V> {
        &mut self.vertex_buffer
    }

    #[inline]
    pub fn index_buffer(&self) -> &Buffer<I> {
        &self.index_buffer
    }

    #[inline]
    pub fn index_buffer_mut(&mut self) -> &mut Buffer<I> {
        &mut self.index_buffer
    }

    pub fn unwrap(mut self) -> (Buffer<V>, Buffer<I>) {
        unsafe {
            self.destroy_in_place();
            let buffers = (ptr::read(&self.vertex_buffer), ptr::read(&self.index_buffer));

            mem::forget(self);

            buffers
        }
    }

    /// Destroy the VAO **without** recursively dropping the contained vertex and index buffers
    unsafe fn destroy_in_place(&mut self) {
        let mut raw_vao = mem::uninitialized();
        mem::swap(&mut raw_vao, &mut self.raw);
        raw_vao.delete(&**self.vertex_buffer.state());
    }
}

impl<V: TypeGroup> VertexArrayObj<V, ()> {
    #[inline]
    pub fn new_noindex(vertex_buffer: Buffer<V>) -> VertexArrayObj<V, ()> {
        let index_buffer: Buffer<()> = Buffer::with_size(BufferUsage::StaticDraw, 0, vertex_buffer.state().clone());
        VertexArrayObj::new(vertex_buffer, index_buffer)
    }
}

impl VAOTarget {
    #[inline]
    pub fn new() -> VAOTarget {
        VAOTarget(RawVAOTarget::new())
    }

    #[inline]
    pub unsafe fn bind<'a, V, I>(&'a self, vao: &'a VertexArrayObj<V, I>) -> BoundVAO<'a, V, I>
        where V: TypeGroup,
              I: Index
    {
        BoundVAO {
            vao,
            _bind: self.0.bind(&vao.raw, &vao.vertex_buffer, &vao.index_buffer, &vao.vertex_buffer.state().gl)
        }
    }
}

impl<'a, V: TypeGroup, I: Index> BoundVAO<'a, V, I> {
    pub fn vao(&self) -> &VertexArrayObj<V, I> {
        self.vao
    }
}

impl<V: TypeGroup, I: Index> Drop for VertexArrayObj<V, I> {
    fn drop(&mut self) {
        unsafe{ self.destroy_in_place() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::{CONTEXT_STATE, TestVertex};

    quickcheck!{
        fn make_vao_noindex(buffer_data: Vec<TestVertex>) -> () {
            CONTEXT_STATE.with(|context_state| {
                let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &buffer_data, context_state.clone());
                let _vao = VertexArrayObj::new_noindex(vertex_buffer);
            });
        }
    }
}
