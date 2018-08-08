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

use ContextState;
use self::raw::*;
use colors::ColorFormat;

use glsl::{TypeUniform, TypeTag, TypeBasicTag};
use seal::Sealed;

use std::mem;
use std::rc::Rc;

use num_traits::{PrimInt, Signed};

pub use self::raw::{targets, Dims, DimsSquare, DimsTag, Swizzle, Filter, MipSelector, Image, TextureType, ArrayTextureType};

use cgmath::{Point1, Point2, Point3};
use cgmath_geometry::DimsBox;


pub struct Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    raw: RawTexture<C, T>,
    state: Rc<ContextState>
}

#[derive(Debug, Clone)]
pub enum TexCreateError<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    DimsExceedMax {
        requested: T::Dims,
        max: T::Dims
    }
}

pub(crate) struct SamplerUnits(RawSamplerUnits);
pub(crate) struct BoundTexture<'a, C, T>(RawBoundTexture<'a, C, T>)
    where C: ColorFormat,
          T: TextureType<C>;

impl<C, T> Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C, MipSelector=u8>
{
    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Result<Texture<C, T>, TexCreateError<C, T>>
        where I: Image<'a, C, T>,
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
            // it's using the last texture, congratulations! You have waaay to many textures. Scale it back yo)
            let last_unit = state.sampler_units.0.num_units() - 1;
            let mut bind = unsafe{ state.sampler_units.0.bind_mut(last_unit, &mut raw, &state.gl) };

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

impl<C, T> Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn new(dims: T::Dims, mips: T::MipSelector, state: Rc<ContextState>) -> Result<Texture<C, T>, TexCreateError<C, T>> {
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
            let last_unit = state.sampler_units.0.num_units() - 1;
            let mut bind = unsafe{ state.sampler_units.0.bind_mut(last_unit, &mut raw, &state.gl) };
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
        where I: Image<'a, C, T>
    {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.sub_image(level, offset, sub_dims, image);
    }

    #[inline]
    pub fn swizzle_mask(&mut self, r: Swizzle, g: Swizzle, b: Swizzle, a: Swizzle) {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.swizzle_mask(r, g, b, a);
    }

    #[inline]
    pub fn filtering(&mut self, minify: Filter, magnify: Filter) {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.filtering(minify, magnify);
    }

    #[inline]
    pub fn max_anisotropy(&mut self, max_anisotropy: f32) {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.max_anisotropy(max_anisotropy);
    }

    #[inline]
    pub fn texture_wrap_s(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims1D
    {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_s(texture_wrap);
    }

    #[inline]
    pub fn texture_wrap_t(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims2D
    {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_t(texture_wrap);
    }

    #[inline]
    pub fn texture_wrap_r(&mut self, texture_wrap: TextureWrap)
        where T::Dims: Dims3D
    {
        let last_unit = self.state.sampler_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.sampler_units.0.bind_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.texture_wrap_r(texture_wrap);
    }
}

impl SamplerUnits {
    #[inline]
    pub fn new(gl: &Gl) -> SamplerUnits {
        SamplerUnits(RawSamplerUnits::new(gl))
    }

    #[inline]
    pub unsafe fn bind<'a, C, T>(&'a self, unit: u32, tex: &'a Texture<C, T>, gl: &'a Gl) -> BoundTexture<C, T>
        where C: ColorFormat,
              T: TextureType<C>
    {
        BoundTexture(self.0.bind(unit, &tex.raw, gl))
    }
}


impl<C, T> Drop for Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    fn drop(&mut self) {
        unsafe {
            let mut raw_tex = mem::uninitialized();
            mem::swap(&mut raw_tex, &mut self.raw);
            raw_tex.delete(&self.state);
        }
    }
}

impl<'a, C, T> Sealed for &'a Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C> {}

macro_rules! texture_type_uniform {
    () => ();
    (
        impl &Texture<C, $texture_type:ty> = ($tag_ident:ident, $u_tag_ident:ident, $i_tag_ident:ident);
        $($rest:tt)*
    ) => {
        texture_type_uniform!{
            default impl &Texture<C, $texture_type> = $tag_ident;
            default impl &Texture<C, $texture_type> = $u_tag_ident where C::Scalar: PrimInt;
            impl &Texture<C, $texture_type> = $i_tag_ident where C::Scalar: PrimInt, Signed;
            $($rest)*
        }
    };
    (
        impl &Texture<C, $texture_type:ty> = $tag_ident:ident
            $(where C::Scalar: $($scalar_trait:path),+)*;
        $($rest:tt)*
    ) => {
        unsafe impl<'a, C> TypeUniform for &'a Texture<C, $texture_type>
            where C: ColorFormat,
                $(C::Scalar: $($scalar_trait +)+)*
        {
            #[inline]
            fn uniform_tag() -> TypeTag {TypeTag::Single(TypeBasicTag::$tag_ident)}
        }

        texture_type_uniform!{$($rest)*}
    };
    (
        default impl &Texture<C, $texture_type:ty> = $tag_ident:ident
            $(where C::Scalar: $($scalar_trait:path),+)*;
        $($rest:tt)*
    ) => {
        unsafe impl<'a, C> TypeUniform for &'a Texture<C, $texture_type>
            where C: ColorFormat,
                $(C::Scalar: $($scalar_trait +)+)*
        {
            #[inline]
            default fn uniform_tag() -> TypeTag {TypeTag::Single(TypeBasicTag::$tag_ident)}
        }

        texture_type_uniform!($($rest)*);
    };
}

texture_type_uniform!{
    impl &Texture<C, targets::SimpleTex<DimsBox<Point1<u32>>>> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<C, targets::SimpleTex<DimsBox<Point2<u32>>>> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<C, targets::SimpleTex<DimsBox<Point3<u32>>>> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<C, targets::CubemapTex> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<C, targets::RectTex> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<C, targets::MultisampleTex> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
}
