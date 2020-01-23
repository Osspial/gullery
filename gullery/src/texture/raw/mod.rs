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

pub mod types;

use crate::{
    geometry::{Dimension, D1, D2, D3},
    gl::{self, types::*, Gl},
    geometry::{GLVec2, GLVec3, NonNormalized},
    Handle,
};

use crate::{
    image_format::{ConcreteImageFormat, FormatAttributes, ImageFormat, ImageFormatRenderable},
    ContextState,
};

use std::{
    cell::Cell,
    iter,
    marker::PhantomData,
    mem,
    ops::{Deref, Range},
    ptr,
};

use super::sample_parameters::*;

#[repr(C)]
pub struct RawTexture<D, T>
where
    D: Dimension<u32>,
    T: ?Sized + TextureType<D>,
{
    handle: Handle,
    dims: T::Dims,
    num_mips: T::MipSelector,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawSampler {
    handle: Handle,
    _sendsync_optout: PhantomData<*const ()>,
}

// pub struct RawTextureArray<T: TextureType>
//     where T: TextureType<ArrayLayerSelector=usize>
// {
//     handle: GLuint,
//     dims: D,
//     mips: u8,
//     size: usize
// }

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct ImageUnit {
    texture: Cell<Option<Handle>>,
    sampler: Cell<Option<Handle>>,
}

pub struct RawImageUnits {
    /// The number of image units is never going to change, so storing this as `Box<[]>` means we
    /// don't have to deal with storing the capacity.
    image_units: Box<[ImageUnit]>,
    active_unit: Cell<u32>,
}

#[repr(C)]
pub struct RawBoundTexture<'a, D, T>
where
    D: Dimension<u32>,
    T: 'a + ?Sized + TextureType<D>,
{
    tex: &'a RawTexture<D, T>,
    gl: &'a Gl,
}

#[repr(C)]
pub struct RawBoundTextureMut<'a, D, T>
where
    D: Dimension<u32>,
    T: 'a + ?Sized + TextureType<D>,
{
    tex: &'a mut RawTexture<D, T>,
    gl: &'a Gl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimsSquare {
    pub side: u32,
}

pub trait Dims: 'static + Copy {
    type Offset: Copy;
    fn width(self) -> u32;
    fn height(self) -> u32;
    fn depth(self) -> u32;
    fn num_pixels(self) -> u32;
    fn max_size(state: &ContextState) -> Self;
    fn mip_dims(self, mip_level: GLint) -> Self;
}

pub trait DimsArray: Dims {
    fn max_size_array(state: &ContextState) -> Self;
    fn mip_dims_array(self, mip_level: GLint) -> Self;
}

pub unsafe trait TextureType<D: Dimension<u32>>: 'static {
    type Dims: Dims;
    type MipSelector: MipSelector;
    type Samples: Samples;
    type Format: ?Sized + ImageFormat;

    type Dyn: ?Sized + TextureType<D, MipSelector = Self::MipSelector>;

    const BIND_TARGET: GLenum;
    fn max_size(state: &ContextState) -> Self::Dims;
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims;
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        samples: Self::Samples,
        data_ptr: *const GLvoid,
        data_len: GLsizei,
    ) where
        Self::Format: ConcreteImageFormat;
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei,
    ) where
        Self::Format: ConcreteImageFormat;
}

pub unsafe trait TextureTypeRenderable<D: Dimension<u32>>: TextureType<D> {
    type DynRenderable: ?Sized + TextureType<D, MipSelector = Self::MipSelector>;
}

pub unsafe trait TextureTypeBasicImage<D: Dimension<u32>>: TextureType<D> {}

// pub unsafe trait ArrayTextureType: TextureType {
//     const ARRAY_BIND_TARGET: GLenum;
// }

pub trait MipSelector: Copy {
    type IterLess: Iterator<Item = Self>;

    fn base() -> Self;
    fn to_glint(self) -> GLint;
    fn iter_less(self) -> Self::IterLess;
    fn try_increment(self) -> Self;
}
pub trait Image<'a, D, T>: Copy + Sized
where
    D: Dimension<u32>,
    T: TextureType<D>,
    T::Format: Sized,
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, for_each: F);
    fn variants_static<F: FnMut(GLenum)>(for_each: F);
}
pub trait Samples: Copy {
    fn samples(self) -> Option<GLsizei>;
}

impl Samples for () {
    fn samples(self) -> Option<GLsizei> {
        None
    }
}

impl Samples for u8 {
    fn samples(self) -> Option<GLsizei> {
        Some(self as GLsizei)
    }
}

impl<D, T> RawTexture<D, T>
where
    D: Dimension<u32>,
    T: ?Sized + TextureType<D>,
{
    pub fn new(dims: T::Dims, gl: &Gl) -> RawTexture<D, T> {
        unsafe {
            let mut handle = 0;
            gl.GenTextures(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawTexture {
                handle,
                dims,
                num_mips: mem::zeroed(),
                _sendsync_optout: PhantomData,
            }
        }
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.dims
    }

    #[inline]
    pub fn num_mips(&self) -> u8 {
        match mem::size_of::<T::MipSelector>() {
            // If the mip selector is (), then the number of mips is defined as 1
            0 => 1,
            // otherwise, if it's a u8, it's just the number
            1 => self.num_mips.to_glint() as u8,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        state.gl.DeleteTextures(1, &self.handle.get());
        state
            .image_units
            .0
            .unbind_texture(self.handle, T::BIND_TARGET, &state.gl);
    }
}

impl RawSampler {
    pub fn new(gl: &Gl) -> RawSampler {
        unsafe {
            let mut handle = 0;
            gl.GenSamplers(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawSampler {
                handle,
                _sendsync_optout: PhantomData,
            }
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        state.gl.DeleteSamplers(1, &self.handle.get());
        state
            .image_units
            .0
            .unbind_sampler_from_all(self.handle, &state.gl);
    }
}

impl RawImageUnits {
    pub fn new(gl: &Gl) -> RawImageUnits {
        let mut max_tex_units = 0;
        unsafe {
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut max_tex_units);
            gl.PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }
        assert!(0 <= max_tex_units);

        RawImageUnits {
            image_units: vec![ImageUnit::default(); max_tex_units as usize].into_boxed_slice(),
            active_unit: Cell::new(0),
        }
    }

    // #[inline]
    // pub fn active_unit(&self) -> u32 {
    //     self.active_unit.get()
    // }

    #[inline]
    pub fn num_units(&self) -> u32 {
        self.image_units.len() as u32
    }

    #[inline]
    pub unsafe fn bind_texture<'a, D, T>(
        &'a self,
        unit: u32,
        tex: &'a RawTexture<D, T>,
        gl: &'a Gl,
    ) -> RawBoundTexture<'a, D, T>
    where
        D: Dimension<u32>,
        T: 'a + ?Sized + TextureType<D>,
    {
        let max_unit = self.image_units.len() as u32 - 1;

        if max_unit < unit {
            panic!(
                "attempted to bind to unavailable sampler unit {}; highest unit is {}",
                unit, max_unit
            );
        }

        if unit != self.active_unit.get() {
            self.active_unit.set(unit);
            gl.ActiveTexture(gl::TEXTURE0 + unit);
        }

        let active_image_unit = &self.image_units[unit as usize];
        if active_image_unit.texture.get() != Some(tex.handle) {
            active_image_unit.texture.set(Some(tex.handle));
            gl.BindTexture(T::BIND_TARGET, tex.handle.get());
        }

        RawBoundTexture { tex, gl }
    }

    #[inline]
    pub unsafe fn bind_texture_mut<'a, D, T>(
        &'a self,
        unit: u32,
        tex: &'a mut RawTexture<D, T>,
        gl: &'a Gl,
    ) -> RawBoundTextureMut<'a, D, T>
    where
        D: Dimension<u32>,
        T: 'a + ?Sized + TextureType<D>,
    {
        self.bind_texture(unit, tex, gl);
        RawBoundTextureMut { tex, gl }
    }

    #[inline]
    pub unsafe fn bind_sampler(&self, unit: u32, sampler: &RawSampler, gl: &Gl) {
        let max_unit = self.image_units.len() as u32 - 1;

        if max_unit < unit {
            panic!(
                "attempted to bind to unavailable sampler unit {}; highest unit is {}",
                unit, max_unit
            );
        }

        let active_image_unit = &self.image_units[unit as usize];
        if active_image_unit.sampler.get() != Some(sampler.handle) {
            gl.BindSampler(unit, sampler.handle.get());
        }
    }

    unsafe fn unbind_texture(&self, handle: Handle, target: GLuint, gl: &Gl) {
        for (unit_index, unit) in self.image_units.iter().enumerate() {
            if unit.texture.get() == Some(handle) {
                gl.ActiveTexture(gl::TEXTURE0 + unit_index as GLuint);
                gl.BindTexture(target, 0);
                unit.texture.set(None);
                self.active_unit.set(unit_index as GLuint);
            }
        }
    }

    unsafe fn unbind_sampler_from_all(&self, handle: Handle, gl: &Gl) {
        for (unit_index, unit) in self.image_units.iter().enumerate() {
            if unit.sampler.get() == Some(handle) {
                gl.BindSampler(unit_index as GLuint, 0);
                unit.sampler.set(None);
            }
        }
    }

    pub unsafe fn unbind_sampler_from_unit(&self, unit_index: GLuint, gl: &Gl) {
        let unit = &self.image_units[unit_index as usize];
        gl.BindSampler(unit_index, 0);
        unit.sampler.set(None);
    }
}

impl<'a, D, T> RawBoundTexture<'a, D, T>
where
    D: Dimension<u32>,
    T: TextureType<D>,
{
    pub fn raw_tex(&self) -> &RawTexture<D, T> {
        &self.tex
    }
}

pub trait ParameterUploader {
    fn gl(&self) -> &Gl;
    fn float(&self, pname: GLenum, param: f32);
    fn int(&self, pname: GLenum, param: i32);

    #[inline]
    fn upload_parameters(
        &self,
        parameters: SampleParameters,
        old_parameters_cell: &Cell<SampleParameters>,
    ) {
        let old_parameters = old_parameters_cell.get();

        macro_rules! upload {
            ($($param:ident => $expr:expr;)+) => {
                let SampleParameters {
                    $($param),+
                } = parameters;

                $(
                    if $param != old_parameters.$param {
                        $expr;
                    }
                )+
            };
        }

        upload! {
            filter_min => self.int(gl::TEXTURE_MIN_FILTER, GLenum::from(filter_min) as i32);
            filter_mag => self.int(gl::TEXTURE_MAG_FILTER, GLenum::from(filter_mag) as i32);
            anisotropy_max => {
                let mut max_ma = 256.0; // arbitrarily large number
                unsafe{ self.gl().GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_ma) };

                self.float(gl::TEXTURE_MAX_ANISOTROPY_EXT, anisotropy_max.max(1.0).min(max_ma));
            };
            texture_wrap => {
                self.int(gl::TEXTURE_WRAP_S, GLenum::from(texture_wrap.s) as i32);
                self.int(gl::TEXTURE_WRAP_T, GLenum::from(texture_wrap.t) as i32);
                self.int(gl::TEXTURE_WRAP_R, GLenum::from(texture_wrap.r) as i32);
            };
            lod => {
                self.float(gl::TEXTURE_MIN_LOD, lod.min);
                self.float(gl::TEXTURE_MAX_LOD, lod.max);
                self.float(gl::TEXTURE_LOD_BIAS, lod.bias);
            };
        }
        old_parameters_cell.set(parameters);
    }
}

impl<'a, D, T> RawBoundTextureMut<'a, D, T>
where
    D: Dimension<u32>,
    T: TextureType<D>,
{
    pub fn alloc_image<'b, I>(
        &mut self,
        level: T::MipSelector,
        samples: T::Samples,
        image: Option<I>,
    ) where
        I: Image<'b, D, T>,
        T::Format: ConcreteImageFormat,
    {
        unsafe {
            let mip_level = level.to_glint();

            if mip_level >= self.tex.num_mips() as GLint {
                self.tex.num_mips = level.try_increment();
                self.gl
                    .TexParameteri(T::BIND_TARGET, gl::TEXTURE_MAX_LEVEL, mip_level);
            }

            let mip_dims = T::mip_dims(self.tex.dims(), level);
            let num_blocks_expected = T::Format::blocks_for_dims(GLVec3::new(
                mip_dims.width(),
                mip_dims.height(),
                mip_dims.depth(),
            ));

            match image {
                Some(image_data) => image_data.variants(|image_bind, data| {
                    let num_blocks = data.len();
                    if num_blocks == num_blocks_expected {
                        let data_bytes_len = data.len() * mem::size_of::<T::Format>();
                        T::alloc_image(
                            self.gl,
                            image_bind,
                            mip_dims,
                            level,
                            samples,
                            data.as_ptr() as *const GLvoid,
                            data_bytes_len as GLsizei,
                        );
                    } else {
                        panic!(
                            "Mismatched image size; expected {} blocks, found {} blocks",
                            num_blocks_expected, num_blocks
                        );
                    }
                }),
                None => I::variants_static(|image_bind| {
                    T::alloc_image(
                        self.gl,
                        image_bind,
                        mip_dims,
                        level,
                        samples,
                        ptr::null(),
                        0,
                    )
                }),
            }

            assert_eq!(0, self.gl.GetError());
        }
    }

    pub fn sub_image<'b, I>(
        &mut self,
        level: T::MipSelector,
        offset: <T::Dims as Dims>::Offset,
        sub_dims: T::Dims,
        image: I,
    ) where
        I: Image<'b, D, T>,
        T::Format: ConcreteImageFormat,
    {
        unsafe {
            let mip_level = level.to_glint();

            if mip_level >= self.tex.num_mips() as GLint {
                self.tex.num_mips = level.try_increment();
                self.gl
                    .TexParameteri(T::BIND_TARGET, gl::TEXTURE_MAX_LEVEL, mip_level);
            }

            let dims = self.tex.dims();
            let num_blocks_expected =
                T::Format::blocks_for_dims(GLVec3::new(dims.width(), dims.height(), dims.depth()));

            image.variants(|image_bind, data| {
                let num_blocks = data.len();
                if num_blocks == num_blocks_expected {
                    let data_bytes_len = data.len() * mem::size_of::<T::Format>();
                    T::sub_image(
                        self.gl,
                        image_bind,
                        offset,
                        sub_dims,
                        level,
                        data.as_ptr() as *const GLvoid,
                        data_bytes_len as GLsizei,
                    );
                } else {
                    panic!(
                        "Mismatched image size; expected {} blocks, found {} blocks",
                        num_blocks_expected, num_blocks
                    );
                }
            });

            assert_eq!(0, self.gl.GetError());
        }
    }
}

impl<'a, D, T> RawBoundTextureMut<'a, D, T>
where
    D: Dimension<u32>,
    T: ?Sized + TextureType<D>,
{
    #[inline]
    pub fn swizzle_read(&mut self, r: Swizzle, g: Swizzle, b: Swizzle, a: Swizzle) {
        let mask = [
            GLenum::from(r) as i32,
            GLenum::from(g) as i32,
            GLenum::from(b) as i32,
            GLenum::from(a) as i32,
        ];
        unsafe {
            self.gl
                .TexParameteriv(T::BIND_TARGET, gl::TEXTURE_SWIZZLE_RGBA, mask.as_ptr())
        };
    }
}

impl<'a, D, T> ParameterUploader for RawBoundTexture<'a, D, T>
where
    D: Dimension<u32>,
    T: ?Sized + TextureType<D>,
{
    #[inline]
    fn gl(&self) -> &Gl {
        self.gl
    }
    #[inline]
    fn float(&self, pname: GLenum, param: f32) {
        unsafe { self.gl.TexParameterf(T::BIND_TARGET, pname, param) };
    }
    #[inline]
    fn int(&self, pname: GLenum, param: i32) {
        unsafe { self.gl.TexParameteri(T::BIND_TARGET, pname, param) };
    }
}

impl<'a> ParameterUploader for (&'a Gl, &'a RawSampler) {
    #[inline]
    fn gl(&self) -> &Gl {
        self.0
    }
    #[inline]
    fn float(&self, pname: GLenum, param: f32) {
        unsafe { self.0.SamplerParameterf(self.1.handle.get(), pname, param) };
    }
    #[inline]
    fn int(&self, pname: GLenum, param: i32) {
        unsafe { self.0.SamplerParameteri(self.1.handle.get(), pname, param) };
    }
}

impl<'a, D, T> Deref for RawBoundTextureMut<'a, D, T>
where
    D: Dimension<u32>,
    T: TextureType<D>,
{
    type Target = RawBoundTexture<'a, D, T>;
    #[inline]
    fn deref(&self) -> &RawBoundTexture<'a, D, T> {
        unsafe { &*(self as *const _ as *const RawBoundTexture<'a, D, T>) }
    }
}

impl MipSelector for () {
    type IterLess = iter::Once<()>;

    #[inline]
    fn base() {}
    #[inline]
    fn to_glint(self) -> GLint {
        0
    }
    #[inline]
    fn iter_less(self) -> iter::Once<()> {
        iter::once(self)
    }
    #[inline]
    fn try_increment(self) {}
}
impl MipSelector for u8 {
    type IterLess = Range<u8>;

    #[inline]
    fn base() -> u8 {
        0
    }
    #[inline]
    fn to_glint(self) -> GLint {
        self as GLint
    }
    #[inline]
    fn iter_less(self) -> Range<u8> {
        0..self
    }
    #[inline]
    fn try_increment(self) -> u8 {
        self + 1
    }
}

impl DimsSquare {
    #[inline]
    pub fn new(side: u32) -> DimsSquare {
        DimsSquare { side }
    }
}

impl Dims for u32 {
    type Offset = Self;

    #[inline]
    fn width(self) -> u32 {
        self
    }
    #[inline]
    fn height(self) -> u32 {
        1
    }
    #[inline]
    fn depth(self) -> u32 {
        1
    }
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width()
    }
    #[inline]
    fn max_size(state: &ContextState) -> u32 {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            size as u32
        }
    }
    fn mip_dims(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        self / dim_divisor
    }
}

impl Dims for GLVec2<u32, NonNormalized> {
    type Offset = Self;
    #[inline]
    fn width(self) -> u32 {
        self.x
    }
    #[inline]
    fn height(self) -> u32 {
        self.y
    }
    #[inline]
    fn depth(self) -> u32 {
        1
    }
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width() * self.height()
    }
    #[inline]
    fn max_size(state: &ContextState) -> Self {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            GLVec2::new(size as u32, size as u32)
        }
    }
    fn mip_dims(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        GLVec2::new(self.width() / dim_divisor, self.height() / dim_divisor)
    }
}
impl DimsArray for GLVec2<u32, NonNormalized> {
    #[inline]
    fn max_size_array(state: &ContextState) -> Self {
        unsafe {
            let (mut size, mut array_size) = (0, 0);
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            state
                .gl
                .GetIntegerv(gl::MAX_ARRAY_TEXTURE_LAYERS, &mut array_size);
            GLVec2::new(size as u32, array_size as u32)
        }
    }
    fn mip_dims_array(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        GLVec2::new(self.width() / dim_divisor, self.height())
    }
}

impl Dims for DimsSquare {
    type Offset = GLVec2<u32, NonNormalized>;
    #[inline]
    fn width(self) -> u32 {
        self.side
    }
    #[inline]
    fn height(self) -> u32 {
        self.side
    }
    #[inline]
    fn depth(self) -> u32 {
        1
    }
    #[inline]
    fn num_pixels(self) -> u32 {
        self.side * self.side
    }
    #[inline]
    fn max_size(state: &ContextState) -> DimsSquare {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            DimsSquare::new(size as u32)
        }
    }
    fn mip_dims(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        DimsSquare::new(self.side / dim_divisor)
    }
}
impl Dims for GLVec3<u32, NonNormalized> {
    type Offset = Self;
    #[inline]
    fn width(self) -> u32 {
        self.x
    }
    #[inline]
    fn height(self) -> u32 {
        self.y
    }
    #[inline]
    fn depth(self) -> u32 {
        self.z
    }
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width() * self.height() * self.depth()
    }
    #[inline]
    fn max_size(state: &ContextState) -> Self {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_3D_TEXTURE_SIZE, &mut size);
            GLVec3::new(size as u32, size as u32, size as u32)
        }
    }
    fn mip_dims(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        GLVec3::new(
            self.width() / dim_divisor,
            self.height() / dim_divisor,
            self.depth() / dim_divisor,
        )
    }
}
impl DimsArray for GLVec3<u32, NonNormalized> {
    #[inline]
    fn max_size_array(state: &ContextState) -> GLVec3<u32, NonNormalized> {
        unsafe {
            let (mut size, mut array_size) = (0, 0);
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            state
                .gl
                .GetIntegerv(gl::MAX_ARRAY_TEXTURE_LAYERS, &mut array_size);
            GLVec3::new(size as u32, size as u32, array_size as u32)
        }
    }
    fn mip_dims_array(self, mip_level: GLint) -> Self {
        let dim_divisor = 2u32.pow(mip_level as u32);
        GLVec3::new(
            self.width() / dim_divisor,
            self.height() / dim_divisor,
            self.depth(),
        )
    }
}
impl<'a, D, T> Image<'a, D, T> for &'a [T::Format]
where
    D: Dimension<u32>,
    T: TextureTypeBasicImage<D>,
    T::Format: Sized,
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, mut for_each: F) {
        for_each(T::BIND_TARGET, self);
    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::BIND_TARGET);
    }
}
impl<'a, D, T> Image<'a, D, T> for !
where
    D: Dimension<u32>,
    T: TextureType<D>,
    T::Format: Sized,
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, _: F) {}
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::BIND_TARGET);
    }
}
