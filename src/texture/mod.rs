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

#[macro_use]
pub mod sample_parameters;
mod raw;

use gl::Gl;
use gl::types::*;

use {ContextState, GLObject, Handle};
use self::raw::*;
use self::sample_parameters::*;
use image_format::{ConcreteImageFormat, ImageFormat, Rgba};

use glsl::{TypeTag, TypeBasicTag, ScalarType};
use uniform::{UniformType, TextureUniformBinder};

use std::{mem, io, fmt};
use std::rc::Rc;
use std::cell::Cell;
use std::error::Error;

pub use self::raw::{
    targets, Dims, DimsSquare, DimsTag, MipSelector, Image, TextureType,
    CubeImage, TextureTypeSingleImage, TextureTypeRenderable
};

use cgmath_geometry::{Dimensionality, D1, D2, D3};


#[repr(C)]
pub struct Texture<D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>,
{
    raw: RawTexture<D, T>,
    state: Rc<ContextState>
}

pub struct Sampler {
    pub sample_parameters: SampleParameters,
    old_sample_parameters: Cell<SampleParameters>,

    raw: RawSampler,
    state: Rc<ContextState>,
}

pub struct SampledTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
{
    pub sampler: &'a Sampler,
    pub texture: &'a Texture<D, T>
}

#[derive(Debug, Clone)]
pub enum TexCreateError<D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>
{
    DimsExceedMax {
        requested: T::Dims,
        max: T::Dims
    }
}

impl<D, T> GLObject for Texture<D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
{
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

impl GLObject for Sampler {
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

pub(crate) struct ImageUnits(RawImageUnits);
pub(crate) struct BoundTexture<'a, D, T>(RawBoundTexture<'a, D, T>)
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>;

impl<D, T> Texture<D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          T::Format: ConcreteImageFormat
{
    pub fn new(dims: T::Dims, mips: T::MipSelector, state: Rc<ContextState>) -> Result<Texture<D, T>, TexCreateError<D, T>> {
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
            let mut bind = unsafe{ state.image_units.0.bind_texture_mut(last_unit, &mut raw, &state.gl) };
            for level in mips.iter_less() {
                bind.alloc_image::<!>(level, None);
            }
        }

        Ok(Texture {
            raw,
            state,
        })
    }

    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Result<Texture<D, T>, TexCreateError<D, T>>
        where T: TextureType<D, MipSelector=u8>,
              I: Image<'a, D, T>,
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
            let mut bind = unsafe{ state.image_units.0.bind_texture_mut(last_unit, &mut raw, &state.gl) };

            for (level, image) in images.into_iter().enumerate() {
                bind.alloc_image(level as u8, Some(image));
            }

            if bind.raw_tex().num_mips() == 0 {
                bind.alloc_image::<!>(0, None);
            }
        }

        Ok(Texture {
            raw,
            state,
        })
    }

    #[inline]
    pub fn sub_image<'a, I>(&mut self, level: T::MipSelector, offset: <T::Dims as Dims>::Offset, sub_dims: T::Dims, image: I)
        where I: Image<'a, D, T>
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_texture_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.sub_image(level, offset, sub_dims, image);
    }
}

impl<D, T> Texture<D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>,
{
    #[inline]
    pub fn num_mips(&self) -> u8 {
        self.raw.num_mips()
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.raw.dims()
    }

    #[inline]
    pub fn swizzle_mask(&mut self, mask: Rgba<Swizzle>) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_texture_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.swizzle_mask(mask.r, mask.g, mask.b, mask.a);
    }

    #[inline]
    pub fn as_dyn(&self) -> &Texture<D, T::Dyn> {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::Dyn>>());
        unsafe{ &*(self as *const Texture<D, T> as *const Texture<D, T::Dyn>) }
    }

    #[inline]
    pub fn as_dyn_mut(&mut self) -> &mut Texture<D, T::Dyn> {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::Dyn>>());
        unsafe{ &mut *(self as *mut Texture<D, T> as *mut Texture<D, T::Dyn>) }
    }

    #[inline]
    pub fn into_dyn(self) -> Texture<D, T::Dyn> {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::Dyn>>());
        let tex = unsafe{ mem::transmute_copy::<Texture<D, T>, Texture<D, T::Dyn>>(&self) };
        mem::forget(self);
        tex
    }

    #[inline]
    pub fn as_dyn_renderable(&self) -> &Texture<D, T::DynRenderable>
        where T: TextureTypeRenderable<D>
    {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::DynRenderable>>());
        unsafe{ &*(self as *const Texture<D, T> as *const Texture<D, T::DynRenderable>) }
    }

    #[inline]
    pub fn as_dyn_renderable_mut(&mut self) -> &mut Texture<D, T::DynRenderable>
        where T: TextureTypeRenderable<D>
    {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::DynRenderable>>());
        unsafe{ &mut *(self as *mut Texture<D, T> as *mut Texture<D, T::DynRenderable>) }
    }

    #[inline]
    pub fn into_dyn_renderable(self) -> Texture<D, T::DynRenderable>
        where T: TextureTypeRenderable<D>
    {
        assert_eq!(mem::size_of::<Texture<D, T>>(), mem::size_of::<Texture<D, T::DynRenderable>>());
        let tex = unsafe{ mem::transmute_copy::<Texture<D, T>, Texture<D, T::DynRenderable>>(&self) };
        mem::forget(self);
        tex
    }
}


impl Sampler {
    pub fn new(context: Rc<ContextState>) -> Sampler {
        Sampler::with_parameters(Default::default(), context)
    }

    pub fn with_parameters(sample_parameters: SampleParameters, context: Rc<ContextState>) -> Sampler {
        Sampler {
            raw: RawSampler::new(&context.gl),
            state: context,
            sample_parameters,
            old_sample_parameters: Cell::new(SampleParameters::default())
        }
    }

    #[inline]
    pub(crate) fn upload_parameters(&self) {
        if self.sample_parameters != self.old_sample_parameters.get() {
            (&self.state.gl, &self.raw).upload_parameters(self.sample_parameters, &self.old_sample_parameters);
        }
    }
}

impl ImageUnits {
    #[inline]
    pub fn new(gl: &Gl) -> ImageUnits {
        ImageUnits(RawImageUnits::new(gl))
    }

    #[inline]
    pub unsafe fn bind<'a, D, T>(&'a self, unit: u32, tex: &'a Texture<D, T>, sampler: Option<&Sampler>, gl: &'a Gl) -> BoundTexture<D, T>
        where D: Dimensionality<u32>,
              T: ?Sized + TextureType<D>,
    {
        let tex_bind = self.0.bind_texture(unit, &tex.raw, gl);
        match sampler {
            Some(sampler) => {
                self.0.bind_sampler(unit, &sampler.raw, gl)
            },
            None => {
                self.0.unbind_sampler_from_unit(unit, gl);
            },
        }
        BoundTexture(tex_bind)
    }
}


impl<D, T> Drop for Texture<D, T>
    where D: Dimensionality<u32>,
          T: ?Sized + TextureType<D>,
{
    fn drop(&mut self) {
        unsafe {
            let mut raw_tex = mem::uninitialized();
            mem::swap(&mut raw_tex, &mut self.raw);
            raw_tex.delete(&self.state);
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            let mut raw_sampler = mem::uninitialized();
            mem::swap(&mut raw_sampler, &mut self.raw);
            raw_sampler.delete(&self.state);
        }
    }
}

macro_rules! texture_type_uniform {
    ($(
        impl &Texture<$d:ty, $texture_type:ty> = ($tag_ident:ident, $u_tag_ident:ident, $i_tag_ident:ident);
    )*) => {$(
        unsafe impl<'a, C> UniformType for &'a Texture<$d, $texture_type>
            where C: ?Sized + ImageFormat,
        {
            #[inline]
            fn uniform_tag() -> TypeTag {
                TypeTag::Single(match C::ScalarType::PRIM_TAG {
                    TypeBasicTag::Float => TypeBasicTag::$tag_ident,
                    TypeBasicTag::Int => TypeBasicTag::$i_tag_ident,
                    TypeBasicTag::Bool |
                    TypeBasicTag::UInt => TypeBasicTag::$u_tag_ident,
                    _ => panic!("Bad scalar type tag")
                })
            }
            #[inline]
            unsafe fn upload(&self, loc: GLint, binder: &mut TextureUniformBinder, gl: &Gl) {
                let unit = binder.bind(self, None, gl);
                gl.Uniform1i(loc, unit as GLint);
            }
        }
    )*};
}

texture_type_uniform!{
    impl &Texture<D1, C> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<D2, C> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<D3, C> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<D2, targets::CubemapTex<C>> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<D2, targets::RectTex<C>> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<D2, targets::MultisampleTex<C>> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
}

unsafe impl<'a, D, T> UniformType for SampledTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          &'a Texture<D, T>: UniformType
{
    #[inline]
    fn uniform_tag() -> TypeTag {
        <&'a Texture<D, T> as UniformType>::uniform_tag()
    }
    #[inline]
    unsafe fn upload(&self, loc: GLint, binder: &mut TextureUniformBinder, gl: &Gl) {
        let unit = binder.bind(self.texture, Some(self.sampler), gl);
        gl.Uniform1i(loc, unit as GLint);
    }
}


impl<D, T> From<TexCreateError<D, T>> for io::Error
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          TexCreateError<D, T>: Send + Sync + fmt::Debug
{
    fn from(err: TexCreateError<D, T>) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl<D: Dimensionality<u32>, T: TextureType<D>> Error for TexCreateError<D, T>
    where TexCreateError<D, T>: fmt::Debug {}

impl<D, T> fmt::Display for TexCreateError<D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            TexCreateError::DimsExceedMax{requested, max} =>
                write!(
                    f,
                    "requested dimensions ({}x{}) exceed OpenGL implementation's maximum dimensions ({}x{})",
                    requested.width(),
                    requested.height(),
                    max.width(),
                    max.height()
                )
        }
    }
}


impl<'a, D, T> Clone for SampledTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
{
    fn clone(&self) -> Self {
        SampledTexture {
            sampler: self.sampler,
            texture: self.texture
        }
    }
}

impl<'a, D, T> Copy for SampledTexture<'a, D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D> {}
