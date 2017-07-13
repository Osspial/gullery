use gl::{self, Gl};
use gl::types::*;

use {ContextState, GLObject, GLSLTyGroup, TyGroupMemberRegistry};
use types::{GLSLType, GLPrim};
use buffers::{Buffer, Index};

use std::mem;
use std::cell::Cell;
use std::marker::PhantomData;

pub struct RawVAO<V: GLSLTyGroup> {
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

pub struct RawBoundVAO<'a, V: GLSLTyGroup>(PhantomData<(&'a RawVAO<V>, *const ())>);

struct VertexAttribBuilder<'a, V: GLSLTyGroup> {
    attrib_index: u32,
    max_attribs: u32,
    gl: &'a Gl,
    _marker: PhantomData<(*const V)>
}


impl<V: GLSLTyGroup> RawVAO<V> {
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
        where V: GLSLTyGroup,
              I: Index
    {
        if self.bound_vao.get() != vao.handle {
            gl.BindVertexArray(vao.handle);
            self.bound_vao.set(vao.handle);
        }

        // Make sure the given buffers are bound and if they aren't, bind them.
        if vbuf.handle() != vao.vbuf.get() {
            gl.BindBuffer(gl::ARRAY_BUFFER, vbuf.handle());
            vao.vbuf.set(vbuf.handle());

            let mut max_attribs = 0;
            gl.GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_attribs);

            // Set the vertex attributes to point to the newly bound vertex buffer
            V::members(VertexAttribBuilder {
                attrib_index: 0,
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

impl<'a, V: GLSLTyGroup> TyGroupMemberRegistry for VertexAttribBuilder<'a, V> {
    type Group = V;

    fn add_member<T: GLSLType>(&mut self, name: &str, get_type: fn(&V) -> &T) {
        let gl = self.gl;
        let vertex = V::default();

        let attrib_ptr = get_type(&vertex) as *const T;
        let attrib_offset = attrib_ptr as *const u8 as isize - &vertex as *const V as *const u8 as isize;

        // Make sure the attribute is actually inside of the type, instead of pointing to a static or smth.
        assert!(attrib_offset >= 0);
        let attrib_offset = attrib_offset as usize;
        assert!(attrib_offset + mem::size_of::<T>() <= mem::size_of::<V>());

        let attrib_size = T::len() * mem::size_of::<T::GLPrim>();
        assert!(attrib_size <= mem::size_of::<T>());

        unsafe {
            if self.attrib_index < self.max_attribs {
                gl.EnableVertexAttribArray(self.attrib_index);
                if T::GLPrim::gl_enum() != gl::DOUBLE {
                    gl.VertexAttribPointer(
                        self.attrib_index,
                        T::len() as GLint,
                        T::GLPrim::gl_enum(),
                        T::GLPrim::normalized() as GLboolean,
                        mem::size_of::<V>() as GLsizei,
                        attrib_offset as *const GLvoid
                    );
                } else {
                    panic!("Attempting to use OpenGL 4 feature")
                    // gl.VertexAttribLPointer(
                    //     self.attrib_index,
                    //     T::len() as GLint,
                    //     T::GLPrim::gl_enum(),
                    //     mem::size_of::<V>() as GLsizei,
                    //     attrib_offset as *const GLvoid
                    // );
                }

                self.attrib_index += 1;
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

impl<V: GLSLTyGroup> GLObject for RawVAO<V> {
    #[inline]
    fn handle(&self) -> GLuint {
        self.handle
    }
}
