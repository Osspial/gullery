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

use gl::{self, Gl};
use gl::types::*;

use {ContextState, GLObject};
use glsl::{TransparentType, Scalar};
use vertex::{Vertex, VertexMemberRegistry};
use buffer::{Buffer, Index};

use std::mem;
use std::cell::Cell;
use std::marker::PhantomData;

pub struct RawVAO<V: Vertex> {
    handle: GLuint,
    /// Handle of the bound vertex buffer
    vbuf: Cell<GLuint>,
    /// Handle of the bound index buffer
    ibuf: Cell<GLuint>,
    _sendsync_optout: PhantomData<(*const (), V)>
}

pub struct RawVAOTarget {
    bound_vao: Cell<GLuint>,
    _sendsync_optout: PhantomData<*const ()>
}

pub struct RawBoundVAO<'a, V: Vertex>(PhantomData<(&'a RawVAO<V>, *const ())>);

struct VertexAttribBuilder<'a, V: Vertex> {
    attrib_loc: u32,
    max_attribs: u32,
    gl: &'a Gl,
    _marker: PhantomData<(*const V)>
}


impl<V: Vertex> RawVAO<V> {
    #[inline]
    pub fn new(gl: &Gl) -> RawVAO<V> {
        unsafe {
            let mut handle = 0;
            gl.GenVertexArrays(1, &mut handle);
            assert_ne!(handle, 0);

            RawVAO {
                handle,
                vbuf: Cell::new(0),
                ibuf: Cell::new(0),
                _sendsync_optout: PhantomData
            }
        }
    }

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.gl.DeleteVertexArrays(1, &self.handle);
            let bound_vao = state.vao_target.0.bound_vao.get();
            if bound_vao == self.handle {
                state.vao_target.0.reset_bind(&state.gl);
            }
        }
    }
}

impl RawVAOTarget {
    #[inline]
    pub fn new() -> RawVAOTarget {
        RawVAOTarget {
            bound_vao: Cell::new(0),
            _sendsync_optout: PhantomData
        }
    }

    #[inline]
    pub unsafe fn bind<'a, V, I>(&'a self, vao: &'a RawVAO<V>, vbuf: &Buffer<V>, ibuf: &Buffer<I>, gl: &Gl) -> RawBoundVAO<'a, V>
        where V: Vertex,
              I: Index
    {
        if self.bound_vao.get() != vao.handle {
            gl.BindVertexArray(vao.handle);
            self.bound_vao.set(vao.handle);
        }

        // Make sure the given buffer are bound and if they aren't, bind them.
        if vbuf.handle() != vao.vbuf.get() {
            gl.BindBuffer(gl::ARRAY_BUFFER, vbuf.handle());
            vao.vbuf.set(vbuf.handle());

            let mut max_attribs = 0;
            gl.GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_attribs);

            // Set the vertex attributes to point to the newly bound vertex buffer
            V::members(VertexAttribBuilder {
                attrib_loc: 0,
                max_attribs: max_attribs as u32,
                gl,
                _marker: PhantomData
            })
        }
        if ibuf.handle() != vao.ibuf.get() {
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibuf.handle());
            vao.ibuf.set(ibuf.handle());
        }

        RawBoundVAO(PhantomData)
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_vao.set(0);
        gl.BindVertexArray(0);
    }
}

impl<'a, V: Vertex> VertexMemberRegistry for VertexAttribBuilder<'a, V> {
    type Group = V;

    fn add_member<T>(&mut self, name: &str, get_type: fn(*const V) -> *const T)
        where T: TransparentType
    {
        let gl = self.gl;
        let vertex = unsafe{ mem::zeroed() };

        let attrib_ptr = get_type(&vertex) as *const T;
        let attrib_offset = attrib_ptr as *const u8 as isize - &vertex as *const V as *const u8 as isize;

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

                    if T::Scalar::GLSL_INTEGER {
                        gl.VertexAttribIPointer(
                            self.attrib_loc + slot,
                            attrib_len as GLint,
                            T::Scalar::GL_ENUM,
                            mem::size_of::<V>() as GLsizei,
                            (attrib_offset + slot_offset) as *const GLvoid
                        );
                    } else if T::Scalar::GL_ENUM != gl::DOUBLE {
                        gl.VertexAttribPointer(
                            self.attrib_loc + slot,
                            attrib_len as GLint,
                            T::Scalar::GL_ENUM,
                            T::Scalar::NORMALIZED as GLboolean,
                            mem::size_of::<V>() as GLsizei,
                            (attrib_offset + slot_offset) as *const GLvoid
                        );
                    } else {
                        panic!("Attempting to use OpenGL 4 feature")
                        // gl.VertexAttribLPointer(
                        //     self.attrib_loc,
                        //     T::len() as GLint,
                        //     T::Scalar::GL_ENUM,
                        //     mem::size_of::<V>() as GLsizei,
                        //     attrib_offset as *const GLvoid
                        // );
                    }
                }

                self.attrib_loc += ty_attrib_slots as u32;
            } else {
                panic!(
                    "Too many attributes on field {}; GL implementation has maximum of {}",
                    name,
                    self.max_attribs
                );
            }
            assert_eq!(0, gl.GetError());
        }
    }
}

impl<V: Vertex> GLObject for RawVAO<V> {
    #[inline]
    fn handle(&self) -> GLuint {
        self.handle
    }
}
