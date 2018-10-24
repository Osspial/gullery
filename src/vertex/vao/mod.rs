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

use {GLObject, Handle, ContextState};
use vertex::{Index, Vertex};
use buffer::Buffer;

use std::{mem, ptr};
use std::rc::Rc;

pub struct VertexArrayObject<V: Vertex, I: Index> {
    raw: RawVAO<V>,
    vertex_buffer: Buffer<V>,
    index_buffer: Option<Buffer<I>>
}

impl<V: Vertex, I: Index> GLObject for VertexArrayObject<V, I> {
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.vertex_buffer.state()
    }
}

pub(crate) struct VAOTarget(RawVAOTarget);
pub(crate) struct BoundVAO<'a, V: Vertex, I: Index> {
    vao: &'a VertexArrayObject<V, I>,
    _bind: RawBoundVAO<'a, V>
}

impl<V: Vertex, I: Index> VertexArrayObject<V, I> {
    pub fn new(vertex_buffer: Buffer<V>, index_buffer: Option<Buffer<I>>) -> VertexArrayObject<V, I> {
        let vertex_buffer_context_ptr = vertex_buffer.state().as_ref() as *const _;
        let index_buffer_context_ptr = index_buffer.as_ref()
                                        .map(|ib| ib.state().as_ref() as *const _)
                                        .unwrap_or(vertex_buffer_context_ptr);

        if vertex_buffer_context_ptr != index_buffer_context_ptr {
            panic!("vertex buffer and index buffer using different contexts");
        }

        VertexArrayObject {
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
    pub fn index_buffer(&self) -> &Option<Buffer<I>> {
        &self.index_buffer
    }

    #[inline]
    pub fn index_buffer_mut(&mut self) -> &mut Option<Buffer<I>> {
        &mut self.index_buffer
    }

    pub fn unwrap(mut self) -> (Buffer<V>, Option<Buffer<I>>) {
        unsafe {
            self.destroy_in_place();
            let buffer = (ptr::read(&self.vertex_buffer), ptr::read(&self.index_buffer));

            mem::forget(self);

            buffer
        }
    }

    /// Destroy the VAO **without** recursively dropping the contained vertex and index buffer
    unsafe fn destroy_in_place(&mut self) {
        let mut raw_vao = mem::uninitialized();
        mem::swap(&mut raw_vao, &mut self.raw);
        raw_vao.delete(&**self.vertex_buffer.state());
    }
}

impl VAOTarget {
    #[inline]
    pub fn new() -> VAOTarget {
        VAOTarget(RawVAOTarget::new())
    }

    #[inline]
    pub unsafe fn bind<'a, V, I>(&'a self, vao: &'a VertexArrayObject<V, I>) -> BoundVAO<'a, V, I>
        where V: Vertex,
              I: Index
    {
        BoundVAO {
            vao,
            _bind: self.0.bind(&vao.raw, &vao.vertex_buffer, &vao.index_buffer, &vao.vertex_buffer.state().gl)
        }
    }
}

impl<'a, V: Vertex, I: Index> BoundVAO<'a, V, I> {
    pub fn vao(&self) -> &VertexArrayObject<V, I> {
        self.vao
    }
}

impl<V: Vertex, I: Index> Drop for VertexArrayObject<V, I> {
    fn drop(&mut self) {
        unsafe{ self.destroy_in_place() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::{CONTEXT_STATE, TestVertex};
    use buffer::BufferUsage;

    quickcheck!{
        fn make_vao_noindex(buffer_data: Vec<TestVertex>) -> () {
            CONTEXT_STATE.with(|context_state| {
                let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &buffer_data, context_state.clone());
                let _vao: VertexArrayObject<TestVertex, !> = VertexArrayObject::new(vertex_buffer, None);
            });
        }
    }
}
