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

use Handle;
use gl::{self, Gl};
use gl::types::*;

use ContextState;
use image_format::{ConcreteImageFormat, ImageFormat, ImageFormatRenderable, FormatAttributes};

use std::{mem, ptr, iter};
use std::cell::Cell;
use std::ops::{Deref, Index, Range};
use std::marker::PhantomData;

use cgmath::{Vector1, Vector2, Vector3};
use cgmath_geometry::{Dimensionality, D1, D2, D3};
use cgmath_geometry::rect::{GeoBox, DimsBox};
use super::sample_parameters::*;

#[repr(C)]
pub struct RawTexture<D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>
{
    handle: Handle,
    dims: T::Dims,
    num_mips: T::MipSelector,
    _sendsync_optout: PhantomData<*const ()>
}

pub struct RawSampler {
    handle: Handle,
    _sendsync_optout: PhantomData<*const ()>
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
    sampler: Cell<Option<Handle>>
}

pub struct RawImageUnits {
    /// The number of image units is never going to change, so storing this as `Box<[]>` means we
    /// don't have to deal with storing the capacity.
    image_units: Box<[ImageUnit]>,
    active_unit: Cell<u32>
}

#[repr(C)]
pub struct RawBoundTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: 'a + ?Sized + TextureType<D>
{
    tex: &'a RawTexture<D, T>,
    gl: &'a Gl
}

#[repr(C)]
pub struct RawBoundTextureMut<'a, D, T>
    where D: Dimensionality<u32>,
          T: 'a + ?Sized + TextureType<D>
{
    tex: &'a mut RawTexture<D, T>,
    gl: &'a Gl
}

#[derive(Debug, PartialEq, Eq)]
pub struct CubeImage<'a, I: ImageFormat> {
    pub pos_x: &'a [I],
    pub neg_x: &'a [I],
    pub pos_y: &'a [I],
    pub neg_y: &'a [I],
    pub pos_z: &'a [I],
    pub neg_z: &'a [I]
}

impl<'a, I: ImageFormat> Clone for CubeImage<'a, I> {
    fn clone(&self) -> CubeImage<'a, I> {
        CubeImage {
            pos_x: self.pos_x,
            neg_x: self.neg_x,
            pos_y: self.pos_y,
            neg_y: self.neg_y,
            pos_z: self.pos_z,
            neg_z: self.neg_z,
        }
    }
}
impl<'a, I: ImageFormat> Copy for CubeImage<'a, I> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimsTag {
    One(DimsBox<D1, u32>),
    Two(DimsBox<D2, u32>),
    Three(DimsBox<D3, u32>)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimsSquare {
    pub side: u32
}

pub trait Dims: 'static + Into<DimsTag> + Copy {
    type Offset: Index<usize, Output=u32>;
    fn width(self) -> u32;
    fn height(self) -> u32;
    fn depth(self) -> u32;
    fn num_pixels(self) -> u32;
    fn max_size(state: &ContextState) -> Self;
    fn max_size_array(_state: &ContextState) -> Option<Self> {None}
}

pub unsafe trait TextureType<D: Dimensionality<u32>>: 'static {
    type MipSelector: MipSelector;
    type Format: ?Sized + ImageFormat;
    type Dims: Dims;

    type Dyn: ?Sized + TextureType<D, MipSelector=Self::MipSelector>;

    const BIND_TARGET: GLenum;
    const IS_ARRAY: bool = false;
}

pub unsafe trait TextureTypeRenderable<D: Dimensionality<u32>>: TextureType<D> {
    type DynRenderable: ?Sized + TextureType<D, MipSelector=Self::MipSelector>;
}

pub unsafe trait TextureTypeSingleImage<D: Dimensionality<u32>>: TextureType<D> {}

// pub unsafe trait ArrayTextureType: TextureType {
//     const ARRAY_BIND_TARGET: GLenum;
// }

pub trait MipSelector: Copy {
    type IterLess: Iterator<Item=Self>;

    fn base() -> Self;
    fn to_glint(self) -> GLint;
    fn iter_less(self) -> Self::IterLess;
    fn try_increment(self) -> Self;
}
pub trait Image<'a, D, T>: Copy + Sized
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          T::Format: Sized
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, for_each: F);
    fn variants_static<F: FnMut(GLenum)>(for_each: F);
}


impl<D, T> RawTexture<D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>
{
    pub fn new(dims: T::Dims, gl: &Gl) -> RawTexture<D, T> {
        unsafe {
            let mut handle = 0;
            gl.GenTextures(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawTexture{
                handle,
                dims,
                num_mips: mem::zeroed(),
                _sendsync_optout: PhantomData
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
            _ => unreachable!()
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.gl.DeleteTextures(1, &self.handle.get());
            state.image_units.0.unbind_texture(self.handle, T::BIND_TARGET, &state.gl);
        }
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
                _sendsync_optout: PhantomData
            }
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.gl.DeleteSamplers(1, &self.handle.get());
            state.image_units.0.unbind_sampler_from_all(self.handle, &state.gl);
        }
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
            active_unit: Cell::new(0)
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
    pub unsafe fn bind_texture<'a, D, T>(&'a self, unit: u32, tex: &'a RawTexture<D, T>, gl: &'a Gl) -> RawBoundTexture<'a, D, T>
        where D: Dimensionality<u32>,
              T: 'a + ?Sized + TextureType<D>
    {
        let max_unit = self.image_units.len() as u32 - 1;

        if max_unit < unit {
            panic!(
                "attempted to bind to unavailable sampler unit {}; highest unit is {}",
                unit,
                max_unit
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

        RawBoundTexture{ tex, gl }
    }

    #[inline]
    pub unsafe fn bind_texture_mut<'a, D, T>(&'a self, unit: u32, tex: &'a mut RawTexture<D, T>, gl: &'a Gl) -> RawBoundTextureMut<'a, D, T>
        where D: Dimensionality<u32>,
              T: 'a + ?Sized + TextureType<D>
    {
        self.bind_texture(unit, tex, gl);
        RawBoundTextureMut{ tex, gl }
    }

    #[inline]
    pub unsafe fn bind_sampler(&self, unit: u32, sampler: &RawSampler, gl: &Gl) {
        let max_unit = self.image_units.len() as u32 - 1;

        if max_unit < unit {
            panic!(
                "attempted to bind to unavailable sampler unit {}; highest unit is {}",
                unit,
                max_unit
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
    where D: Dimensionality<u32>,
          T: TextureType<D>
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
    fn upload_parameters(&self, parameters: SampleParameters, old_parameters_cell: &Cell<SampleParameters>) {
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

        upload!{
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
    where D: Dimensionality<u32>,
          T: TextureType<D>
{
    pub fn alloc_image<'b, I>(&mut self, level: T::MipSelector, image: Option<I>)
        where I: Image<'b, D, T>,
              T::Format: ConcreteImageFormat
    {
        unsafe {
            let mip_level = level.to_glint();

            if mip_level >= self.tex.num_mips() as GLint {
                self.tex.num_mips = level.try_increment();
                self.gl.TexParameteri(T::BIND_TARGET, gl::TEXTURE_MAX_LEVEL, mip_level);
            }



            let dims_exponent = mip_level as u32;
            let (width, height, depth): (GLsizei, GLsizei, GLsizei);
            let dim_divisor = 2u32.pow(dims_exponent);
            match T::IS_ARRAY {
                false => {
                    let (base_width, base_height, base_depth) = self.tex.dims.into().to_tuple();
                    width = (base_width / dim_divisor).max(1) as GLsizei;
                    height = (base_height / dim_divisor).max(1) as GLsizei;
                    depth = (base_depth / dim_divisor).max(1) as GLsizei;
                },
                true => match self.tex.dims.into() {
                    DimsTag::One(_) => panic!("1D Array texture doesn't make sense"),
                    DimsTag::Two(dims) => {
                        width = (dims.width() / dim_divisor).max(1) as GLsizei;
                        height = dims.height() as GLsizei;
                        depth = 1;
                    },
                    DimsTag::Three(dims) => {
                        width = (dims.width() / dim_divisor).max(1) as GLsizei;
                        height = (dims.height() / dim_divisor).max(1) as GLsizei;
                        depth = dims.depth() as GLsizei;
                    }
                }
            }

            let num_pixels_expected = (width * height * depth) as usize;

            let upload_data = |gl: &Gl, image_bind, data, data_len| {
                match T::Format::FORMAT {
                    FormatAttributes::Uncompressed{internal_format, pixel_format, pixel_type} => {
                        match self.tex.dims.into() {
                            DimsTag::One(_) =>
                                gl.TexImage1D(
                                    image_bind, mip_level, internal_format as GLint,
                                    width,
                                    0, pixel_format, pixel_type, data as *const GLvoid
                                ),
                            DimsTag::Two(_) =>
                                gl.TexImage2D(
                                    image_bind, mip_level, internal_format as GLint,
                                    width,
                                    height,
                                    0, pixel_format, pixel_type, data as *const GLvoid
                                ),
                            DimsTag::Three(_) =>
                                gl.TexImage3D(
                                    image_bind, mip_level, internal_format as GLint,
                                    width,
                                    height,
                                    depth,
                                    0, pixel_format, pixel_type, data as *const GLvoid
                                ),
                        }
                    },
                    FormatAttributes::Compressed{internal_format, ..} => {
                        match self.tex.dims.into() {
                            DimsTag::One(_) =>
                                gl.CompressedTexImage1D(
                                    image_bind, mip_level, internal_format,
                                    width,
                                    0, data_len, data as *const GLvoid
                                ),
                            DimsTag::Two(_) =>
                                gl.CompressedTexImage2D(
                                    image_bind, mip_level, internal_format,
                                    width,
                                    height,
                                    0, data_len, data as *const GLvoid
                                ),
                            DimsTag::Three(_) =>
                                gl.CompressedTexImage3D(
                                    image_bind, mip_level, internal_format,
                                    width,
                                    height,
                                    depth,
                                    0, data_len, data as *const GLvoid
                                ),
                        }
                    }
                }
            };

            let pixels_per_unit = match T::Format::FORMAT {
                FormatAttributes::Uncompressed{..} => 1,
                FormatAttributes::Compressed{pixels_per_block, ..} => pixels_per_block
            };

            match image {
                Some(image_data) => image_data.variants(|image_bind, data| {
                    let num_pixels = data.len() * pixels_per_unit;
                    if num_pixels == num_pixels_expected {
                        let data_bytes_len = data.len() * mem::size_of::<T::Format>();
                        upload_data(self.gl, image_bind, data.as_ptr() as *const GLvoid, data_bytes_len as GLsizei);
                    } else {
                        panic!("Mismatched image size; expected {} pixels, found {} pixels", num_pixels_expected, num_pixels);
                    }
                }),
                None => I::variants_static(|image_bind| upload_data(self.gl, image_bind, ptr::null(), 0))
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
    )
        where I: Image<'b, D, T>,
              T::Format: ConcreteImageFormat
    {
        use self::DimsTag::*;

        image.variants(|_, data| {
            let num_pixels = data.len() as u32;
            if num_pixels != sub_dims.num_pixels() {
                panic!("expected {} pixels, found {} pixels", sub_dims.num_pixels(), num_pixels);
            }
        });

        assert!((level.to_glint() as u8) < self.tex.num_mips());
        match (self.tex.dims.into(), sub_dims.into()) {
            (One(tex_dims), One(sub_dims)) => {
                assert!(sub_dims.width() + offset[0] <= tex_dims.width());
            },
            (Two(tex_dims), Two(sub_dims)) => {
                assert!(sub_dims.width() + offset[0] <= tex_dims.width());
                assert!(sub_dims.height() + offset[1] <= tex_dims.height());
            },
            (Three(tex_dims), Three(sub_dims)) => {
                assert!(sub_dims.width() + offset[0] <= tex_dims.width());
                assert!(sub_dims.height() + offset[1] <= tex_dims.height());
                assert!(sub_dims.depth() + offset[2] <= tex_dims.depth());
            },
            _ => unreachable!()
        }

        unsafe {
            match T::Format::FORMAT {
                FormatAttributes::Uncompressed{pixel_format, pixel_type, ..} => match sub_dims.into() {
                    One(dims) => image.variants(|image_bind, data| self.gl.TexSubImage1D(
                        image_bind, level.to_glint(),
                        offset[0] as GLint,
                        dims.width() as GLsizei,
                        pixel_format, pixel_type, data.as_ptr() as *const GLvoid
                    )),
                    Two(dims) => image.variants(|image_bind, data| self.gl.TexSubImage2D(
                        image_bind, level.to_glint(),
                        offset[0] as GLint,
                        offset[1] as GLint,
                        dims.width() as GLsizei,
                        dims.height() as GLsizei,
                        pixel_format, pixel_type, data.as_ptr() as *const GLvoid
                    )),
                    Three(dims) => image.variants(|image_bind, data| self.gl.TexSubImage3D(
                        image_bind, level.to_glint(),
                        offset[0] as GLint,
                        offset[1] as GLint,
                        offset[2] as GLint,
                        dims.width() as GLsizei,
                        dims.height() as GLsizei,
                        dims.depth() as GLsizei,
                        pixel_format, pixel_type, data.as_ptr() as *const GLvoid
                    ))
                },
                FormatAttributes::Compressed{..} => unimplemented!()
            }
        }
    }
}

impl<'a, D, T> RawBoundTextureMut<'a, D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>
{
    #[inline]
    pub fn swizzle_read(&mut self, r: Swizzle, g: Swizzle, b: Swizzle, a: Swizzle) {
        let mask = [
            GLenum::from(r) as i32,
            GLenum::from(g) as i32,
            GLenum::from(b) as i32,
            GLenum::from(a) as i32
        ];
        unsafe{ self.gl.TexParameteriv(T::BIND_TARGET, gl::TEXTURE_SWIZZLE_RGBA, mask.as_ptr()) };
    }
}

impl<'a, D, T> ParameterUploader for RawBoundTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>
{
    #[inline]
    fn gl(&self) -> &Gl { self.gl }
    #[inline]
    fn float(&self, pname: GLenum, param: f32) {
        unsafe{ self.gl.TexParameterf(T::BIND_TARGET, pname, param) };
    }
    #[inline]
    fn int(&self, pname: GLenum, param: i32) {
        unsafe{ self.gl.TexParameteri(T::BIND_TARGET, pname, param) };
    }
}

impl<'a> ParameterUploader for (&'a Gl, &'a RawSampler) {
    #[inline]
    fn gl(&self) -> &Gl { self.0 }
    #[inline]
    fn float(&self, pname: GLenum, param: f32) {
        unsafe{ self.0.SamplerParameterf(self.1.handle.get(), pname, param) };
    }
    #[inline]
    fn int(&self, pname: GLenum, param: i32) {
        unsafe{ self.0.SamplerParameteri(self.1.handle.get(), pname, param) };
    }
}

impl<'a, D, T> Deref for RawBoundTextureMut<'a, D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>
{
    type Target = RawBoundTexture<'a, D, T>;
    #[inline]
    fn deref(&self) -> &RawBoundTexture<'a, D, T> {
        unsafe{ &*(self as *const _ as *const RawBoundTexture<'a, D, T>) }
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
    fn base() -> u8 {0}
    #[inline]
    fn to_glint(self) -> GLint {
        self as GLint
    }
    #[inline]
    fn iter_less(self) -> Range<u8> {
        0..self
    }
    #[inline]
    fn try_increment(self) -> u8 {self + 1}
}

impl DimsSquare {
    #[inline]
    pub fn new(side: u32) -> DimsSquare {
        DimsSquare{ side }
    }
}
impl DimsTag {
    #[inline]
    pub fn to_tuple(self) -> (u32, u32, u32) {
        match self {
            DimsTag::One(dims) => (dims.width(), 1, 1),
            DimsTag::Two(dims) => (dims.width(), dims.height(), 1),
            DimsTag::Three(dims) => (dims.width(), dims.height(), dims.depth())
        }
    }
}

impl Dims for DimsBox<D1, u32> {
    type Offset = Vector1<u32>;

    #[inline] fn width(self) -> u32 {GeoBox::width(&self)}
    #[inline] fn height(self) -> u32 {GeoBox::height(&self)}
    #[inline] fn depth(self) -> u32 {GeoBox::depth(&self)}
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width()
    }
    #[inline]
    fn max_size(state: &ContextState) -> DimsBox<D1, u32> {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            DimsBox::new1(size as u32)
        }
    }
}
impl Dims for DimsBox<D2, u32> {
    type Offset = Vector2<u32>;
    #[inline] fn width(self) -> u32 {GeoBox::width(&self)}
    #[inline] fn height(self) -> u32 {GeoBox::height(&self)}
    #[inline] fn depth(self) -> u32 {GeoBox::depth(&self)}
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width() * self.height()
    }
    #[inline]
    fn max_size(state: &ContextState) -> DimsBox<D2, u32> {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            DimsBox::new2(size as u32, size as u32)
        }
    }
    #[inline]
    fn max_size_array(state: &ContextState) -> Option<DimsBox<D2, u32>> {
        unsafe {
            let (mut size, mut array_size) = (0, 0);
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            state.gl.GetIntegerv(gl::MAX_ARRAY_TEXTURE_LAYERS, &mut array_size);
            Some(DimsBox::new2(size as u32, array_size as u32))
        }
    }
}
impl Dims for DimsSquare {
    type Offset = Vector2<u32>;
    #[inline] fn width(self) -> u32 {self.side}
    #[inline] fn height(self) -> u32 {self.side}
    #[inline] fn depth(self) -> u32 {1}
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
}
impl Dims for DimsBox<D3, u32> {
    type Offset = Vector3<u32>;
    #[inline] fn width(self) -> u32 {GeoBox::width(&self)}
    #[inline] fn height(self) -> u32 {GeoBox::height(&self)}
    #[inline] fn depth(self) -> u32 {GeoBox::depth(&self)}
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width() * self.height() * self.depth()
    }
    #[inline]
    fn max_size(state: &ContextState) -> DimsBox<D3, u32> {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_3D_TEXTURE_SIZE, &mut size);
            DimsBox::new3(size as u32, size as u32, size as u32)
        }
    }
    #[inline]
    fn max_size_array(state: &ContextState) -> Option<DimsBox<D3, u32>> {
        unsafe {
            let (mut size, mut array_size) = (0, 0);
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            state.gl.GetIntegerv(gl::MAX_ARRAY_TEXTURE_LAYERS, &mut array_size);
            Some(DimsBox::new3(size as u32, size as u32, array_size as u32))
        }
    }
}
impl From<DimsBox<D1, u32>> for DimsTag {
    #[inline]
    fn from(dims: DimsBox<D1, u32>) -> DimsTag {
        DimsTag::One(dims)
    }
}
impl From<DimsBox<D2, u32>> for DimsTag {
    #[inline]
    fn from(dims: DimsBox<D2, u32>) -> DimsTag {
        DimsTag::Two(dims)
    }
}
impl From<DimsSquare> for DimsTag {
    #[inline]
    fn from(dims: DimsSquare) -> DimsTag {
        DimsTag::Two(DimsBox::new2(dims.side, dims.side))
    }
}
impl From<DimsBox<D3, u32>> for DimsTag {
    #[inline]
    fn from(dims: DimsBox<D3, u32>) -> DimsTag {
        DimsTag::Three(dims)
    }
}

impl<'a, I> Image<'a, D2, types::CubemapTex<I>> for CubeImage<'a, I>
    where I: ImageFormat
{
    fn variants<F: FnMut(GLenum, &'a [I])>(self, mut for_each: F) {
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_X, self.pos_x);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, self.neg_x);
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, self.pos_y);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, self.neg_y);
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, self.pos_z);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, self.neg_z);
    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_X);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_X);
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_Y);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y);
        for_each(gl::TEXTURE_CUBE_MAP_POSITIVE_Z);
        for_each(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z);
   }
}
impl<'a, D, T> Image<'a, D, T> for &'a [T::Format]
    where D: Dimensionality<u32>,
          T: TextureTypeSingleImage<D>,
          T::Format: Sized
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, mut for_each: F) {
        for_each(T::BIND_TARGET, self);
    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::BIND_TARGET);
   }
}
impl<'a, D, T> Image<'a, D, T> for !
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          T::Format: Sized
{
    fn variants<F: FnMut(GLenum, &'a [T::Format])>(self, _: F) {    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::BIND_TARGET);
   }
}
