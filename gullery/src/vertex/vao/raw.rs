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

use crate::gl::{self, types::*, Gl};

use crate::{
    buffer::Buffer,
    geometry::{Scalar, ScalarBase, ScalarType, TransparentType, TypeTagSingle},
    vertex::{Index, Vertex, VertexMemberRegistry},
    ContextState, GLObject, Handle,
};

use std::{cell::Cell, marker::PhantomData, mem};

pub struct RawVAO<V: Vertex> {
    handle: Handle,
    /// Handle of the bound vertex buffer
    vbuf: Cell<Option<Handle>>,
    /// Handle of the bound index buffer
    ibuf: Cell<Option<Handle>>,
    _sendsync_optout: PhantomData<(*const (), V)>,
}

pub struct RawVAOTarget {
    bound_vao: Cell<Option<Handle>>,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawBoundVAO<'a, V: Vertex>(PhantomData<(&'a RawVAO<V>, *const ())>);

struct VertexAttribBuilder<'a, V: Vertex> {
    attrib_loc: u32,
    max_attribs: u32,
    gl: &'a Gl,
    _marker: PhantomData<*const V>,
}

impl<V: Vertex> RawVAO<V> {
    #[inline]
    pub fn new(gl: &Gl) -> RawVAO<V> {
        unsafe {
            let mut handle = 0;
            gl.GenVertexArrays(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawVAO {
                handle,
                vbuf: Cell::new(None),
                ibuf: Cell::new(None),
                _sendsync_optout: PhantomData,
            }
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        state.gl.DeleteVertexArrays(1, &self.handle.get());
        let bound_vao = state.vao_target.0.bound_vao.get();
        if bound_vao == Some(self.handle) {
            state.vao_target.0.reset_bind(&state.gl);
        }
    }
}

impl RawVAOTarget {
    #[inline]
    pub fn new() -> RawVAOTarget {
        RawVAOTarget {
            bound_vao: Cell::new(None),
            _sendsync_optout: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn bind<'a, V, I>(
        &'a self,
        vao: &'a RawVAO<V>,
        vbuf: &Buffer<V>,
        ibuf: &Option<Buffer<I>>,
        gl: &Gl,
    ) -> RawBoundVAO<'a, V>
    where
        V: Vertex,
        I: Index,
    {
        if self.bound_vao.get() != Some(vao.handle) {
            gl.BindVertexArray(vao.handle.get());
            self.bound_vao.set(Some(vao.handle));
        }

        // Make sure the given buffer are bound and if they aren't, bind them.
        if Some(vbuf.handle()) != vao.vbuf.get() {
            gl.BindBuffer(gl::ARRAY_BUFFER, vbuf.handle().get());
            vao.vbuf.set(Some(vbuf.handle()));

            let mut max_attribs = 0;
            gl.GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_attribs);

            // Set the vertex attributes to point to the newly bound vertex buffer
            V::members(VertexAttribBuilder {
                attrib_loc: 0,
                max_attribs: max_attribs as u32,
                gl,
                _marker: PhantomData,
            })
        }
        let ibuf_handle_opt = ibuf.as_ref().map(|ib| ib.handle());
        if ibuf_handle_opt != vao.ibuf.get() {
            gl.BindBuffer(
                gl::ELEMENT_ARRAY_BUFFER,
                ibuf_handle_opt.map(|h| h.get()).unwrap_or(0),
            );
            vao.ibuf.set(ibuf_handle_opt);
        }

        RawBoundVAO(PhantomData)
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_vao.set(None);
        gl.BindVertexArray(0);
    }
}

impl<'a, V: Vertex> VertexMemberRegistry for VertexAttribBuilder<'a, V> {
    type Group = V;

    fn add_member<T>(&mut self, name: &str, get_type: fn(*const V) -> *const T)
    where
        T: TransparentType,
    {
        let gl = self.gl;
        let vertex = unsafe { mem::zeroed() };

        let attrib_ptr = get_type(&vertex) as *const T;
        let attrib_offset =
            attrib_ptr as *const u8 as isize - &vertex as *const V as *const u8 as isize;

        // Make sure the attribute is actually inside of the type, instead of pointing to a static or smth.
        assert!(attrib_offset >= 0);
        let attrib_offset = attrib_offset as usize;
        assert!(attrib_offset + mem::size_of::<T>() <= mem::size_of::<V>());

        let ty_attrib_slots = T::prim_tag().num_attrib_slots();

        let attrib_len = T::prim_tag().len() / ty_attrib_slots;
        let attrib_size = attrib_len * mem::size_of::<T::Scalar>();
        assert!(attrib_size <= mem::size_of::<T>());

        unsafe {
            if self.attrib_loc < self.max_attribs {
                // Enable all vertex attributes necessary. For matrices, there will be more than one
                // attribute so that's why this loop is needed.
                for slot in 0..ty_attrib_slots as u32 {
                    gl.EnableVertexAttribArray(self.attrib_loc + slot);
                    let slot_offset = slot as usize * attrib_size;

                    match <T::Scalar as Scalar<T::Normalization>>::ScalarType::PRIM_TAG {
                        TypeTagSingle::Float => gl.VertexAttribPointer(
                            self.attrib_loc + slot,
                            attrib_len as GLint,
                            T::Scalar::GL_ENUM,
                            T::Scalar::NORMALIZED as GLboolean,
                            mem::size_of::<V>() as GLsizei,
                            (attrib_offset + slot_offset) as *const GLvoid,
                        ),
                        TypeTagSingle::Int | TypeTagSingle::UInt | TypeTagSingle::Bool =>
                            gl.VertexAttribIPointer(
                                self.attrib_loc + slot,
                                attrib_len as GLint,
                                T::Scalar::GL_ENUM,
                                mem::size_of::<V>() as GLsizei,
                                (attrib_offset + slot_offset) as *const GLvoid,
                            ),
                        // TypeTagSingle::Double => {
                        //     panic!("Attempting to use OpenGL 4 feature")
                        //     gl.VertexAttribLPointer(
                        //         self.attrib_loc,
                        //         T::len() as GLint,
                        //         T::Scalar::GL_ENUM,
                        //         mem::size_of::<V>() as GLsizei,
                        //         attrib_offset as *const GLvoid
                        //     );
                        // },
                        _ => panic!("Invalid scalar type tag"),
                    }
                }

                self.attrib_loc += ty_attrib_slots as u32;
            } else {
                panic!(
                    "Too many attributes on field {}; GL implementation has maximum of {}",
                    name, self.max_attribs
                );
            }
            assert_eq!(0, gl.GetError());
        }
    }
}
