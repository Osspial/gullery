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

use gl::Gl;
use gl::types::*;

use {ContextState, GLObject, Handle};
use self::raw::*;
use color::ImageFormat;

use glsl::{TypeTag, TypeBasicTag, Scalar};
use uniform::{UniformType, TextureUniformBinder};

use std::mem;
use std::rc::Rc;

pub use self::raw::{targets, Dims, DimsSquare, DimsTag, Swizzle, Filter, MipSelector, Image, TextureType, ArrayTextureType};

use cgmath::{Point1, Point2, Point3};
use cgmath_geometry::DimsBox;


pub struct Texture<T>
    where T: TextureType
{
    raw: RawTexture<T>,
    state: Rc<ContextState>
}

#[derive(Debug, Clone)]
pub enum TexCreateError<T>
    where T: TextureType
{
    DimsExceedMax {
        requested: T::Dims,
        max: T::Dims
    }
}

impl<T> GLObject for Texture<T>
    where T: TextureType
{
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

pub(crate) struct ImageUnits(RawImageUnits);
pub(crate) struct BoundTexture<'a, T>(RawBoundTexture<'a, T>)
    where T: TextureType;

impl<T> Texture<T>
    where T: TextureType<MipSelector=u8>
{
    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Result<Texture<T>, TexCreateError<T>>
        where I: Image<'a, T>,
              J: IntoIterator<Item=I>
    {
        let max_size = T::Dims::max_size(&state);
        let (max_width, max_height, max_depth) = max_size.into().to_tuple();
        let (width, height, depth) = dims.into().to_tuple();

        if max_width < width || max_height < height || max_depth < depth {
            return Err(TexCreateError::DimsExceedMax{
                requested: dims,
                max: max_size
            });
        }

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            // We use the last texture unit to make sure that a program never accidentally uses a texture bound
            // during modification.
            //
            // (If you're reading this source code because your program is accidentally using a texture because
            // it's using the last texture, congratulations! You have waaay to many texture. Scale it back yo)
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe{ state.image_units.0.bind_mut(last_unit, &mut raw, &state.gl) };

            for (level, image) in images.into_iter().enumerate() {
                bind.alloc_image(level as u8, Some(image));
            }

            if bind.raw_tex().num_mips() == 0 {
                bind.alloc_image::<!>(0, None);
            }
        }

        Ok(Texture{ raw, state })
    }
}

impl<T> Texture<T>
    where T: TextureType
{
    pub fn new(dims: T::Dims, mips: T::MipSelector, state: Rc<ContextState>) -> Result<Texture<T>, TexCreateError<T>> {
        let max_size = T::Dims::max_size(&state);
        let (max_width, max_height, max_depth) = max_size.into().to_tuple();
        let (width, height, depth) = dims.into().to_tuple();

        if max_width < width || max_height < height || max_depth < depth {
            return Err(TexCreateError::DimsExceedMax{
                requested: dims,
                max: max_size
            });
        }

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe{ state.image_units.0.bind_mut(last_unit, &mut raw, &state.gl) };
            for level in mips.iter_less() {
                bind.alloc_image::<!>(level, None);
            }
        }

        Ok(Texture{ raw, state })
    }

    #[inline]
    pub fn num_mips(&self) -> u8 {
        self.raw.num_mips()
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.raw.dims()
    }

    pub fn sub_image<'a, I>(&mut self, level: T::MipSelector, offset: <T::Dims as Dims>::Offset, sub_dims: T::Dims, image: I)
        where I: Image<'a, T>
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.sub_image(level, offset, sub_dims, image);
    }

    #[inline]
    pub fn swizzle_mask(&mut self, r: Swizzle, g: Swizzle, b: Swizzle, a: Swizzle) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.swizzle_mask(r, g, b, a);
    }

    #[inline]
    pub fn filtering(&mut self, minify: Filter, magnify: Filter) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.filtering(minify, magnify);
    }

    #[inline]
    pub fn max_anisotropy(&mut self, max_anisotropy: f32) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.max_anisotropy(max_anisotropy);
    }

    #[inline]
    pub fn texture_wrap_s(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims1D
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_s(texture_wrap);
    }

    #[inline]
    pub fn texture_wrap_t(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims2D
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_t(texture_wrap);
    }

    #[inline]
    pub fn texture_wrap_r(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims3D
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_r(texture_wrap);
    }
}

impl ImageUnits {
    #[inline]
    pub fn new(gl: &Gl) -> ImageUnits {
        ImageUnits(RawImageUnits::new(gl))
    }

    #[inline]
    pub unsafe fn bind<'a, T>(&'a self, unit: u32, tex: &'a Texture<T>, gl: &'a Gl) -> BoundTexture<T>
        where T: TextureType
    {
        BoundTexture(self.0.bind(unit, &tex.raw, gl))
    }
}


impl<T> Drop for Texture<T>
    where T: TextureType
{
    fn drop(&mut self) {
        unsafe {
            let mut raw_tex = mem::uninitialized();
            mem::swap(&mut raw_tex, &mut self.raw);
            raw_tex.delete(&self.state);
        }
    }
}

macro_rules! texture_type_uniform {
    ($(
        impl &Texture<$texture_type:ty> = ($tag_ident:ident, $u_tag_ident:ident, $i_tag_ident:ident);
    )*) => {$(
        unsafe impl<'a, C> UniformType for &'a Texture<$texture_type>
            where C: ImageFormat
        {
            #[inline]
            fn uniform_tag() -> TypeTag {
                TypeTag::Single(match (C::Scalar::GLSL_INTEGER, C::Scalar::SIGNED) {
                    (false, _) => TypeBasicTag::$tag_ident,
                    (true, true) => TypeBasicTag::$i_tag_ident,
                    (true, false) => TypeBasicTag::$u_tag_ident
                })
            }
            unsafe fn upload(&self, loc: GLint, binder: &mut TextureUniformBinder, gl: &Gl) {
                let unit = binder.bind(self, gl);
                gl.Uniform1i(loc, unit as GLint);
            }
        }
    )*};
}

texture_type_uniform!{
    impl &Texture<targets::SimpleTex<C, DimsBox<Point1<u32>>>> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<targets::SimpleTex<C, DimsBox<Point2<u32>>>> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<targets::SimpleTex<C, DimsBox<Point3<u32>>>> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<targets::CubemapTex<C>> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<targets::RectTex<C>> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<targets::MultisampleTex<C>> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
}
