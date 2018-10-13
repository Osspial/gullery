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
use image_format::{UncompressedFormat, ImageFormat, Rgba};

use glsl::{TypeTag, TypeBasicTag, GLSLScalarType};
use uniform::{UniformType, TextureUniformBinder};

use std::{mem, io, fmt};
use std::rc::Rc;
use std::cell::Cell;
use std::error::Error;

pub use self::raw::{targets, Dims, DimsSquare, DimsTag, MipSelector, Image, TextureType, ArrayTextureType};

use cgmath_geometry::{D1, D2, D3};


pub struct Texture<T, P = SampleParameters>
    where T: TextureType,
          P: IntoSampleParameters
{
    pub sample_parameters: P,
    old_sample_parameters: Cell<P>,

    raw: RawTexture<T>,
    state: Rc<ContextState>
}

pub struct Sampler {
    pub sample_parameters: SampleParameters,
    old_sample_parameters: Cell<SampleParameters>,

    raw: RawSampler,
    state: Rc<ContextState>,
}

pub struct SampledTexture<'a, T, P>
    where T: TextureType,
          P: IntoSampleParameters
{
    pub sampler: &'a Sampler,
    pub texture: &'a Texture<T, P>
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

impl<T, P> GLObject for Texture<T, P>
    where T: TextureType,
          P: IntoSampleParameters
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
pub(crate) struct BoundTexture<'a, T>(RawBoundTexture<'a, T>)
    where T: TextureType;

impl<T, P> Texture<T, P>
    where T: TextureType<MipSelector=u8>,
          P: IntoSampleParameters
{
    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Result<Texture<T, P>, TexCreateError<T>>
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

            sample_parameters: Default::default(),
            old_sample_parameters: Cell::new(Default::default())
        })
    }
}

impl<T, P> Texture<T, P>
    where T: TextureType,
          P: IntoSampleParameters
{
    pub fn new(dims: T::Dims, mips: T::MipSelector, state: Rc<ContextState>) -> Result<Texture<T, P>, TexCreateError<T>> {
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

            sample_parameters: Default::default(),
            old_sample_parameters: Cell::new(Default::default())
        })
    }

    #[inline]
    pub fn num_mips(&self) -> u8 {
        self.raw.num_mips()
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.raw.dims()
    }

    #[inline]
    pub fn sub_image<'a, I>(&mut self, level: T::MipSelector, offset: <T::Dims as Dims>::Offset, sub_dims: T::Dims, image: I)
        where I: Image<'a, T>,
              T::Format: UncompressedFormat
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_texture_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.sub_image(level, offset, sub_dims, image);
    }

    #[inline]
    pub fn swizzle_mask(&mut self, mask: Rgba<Swizzle>) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe{ self.state.image_units.0.bind_texture_mut(last_unit, &mut self.raw, &self.state.gl) };
        bind.swizzle_mask(mask.r, mask.g, mask.b, mask.a);
    }

    #[inline]
    pub(crate) fn upload_parameters(&self) {
        if self.sample_parameters != self.old_sample_parameters.get() {
            let (sp, sp_old) = (
                self.sample_parameters.into_sample_parameters(),
                IntoSampleParameters::into_sample_parameters_cell(&self.old_sample_parameters)
            );
            if let (Some(sp), Some(sp_old)) = (sp, sp_old) {
                let last_unit = self.state.image_units.0.num_units() - 1;
                let bind = unsafe{ self.state.image_units.0.bind_texture(last_unit, &self.raw, &self.state.gl) };
                bind.upload_parameters(sp, sp_old);
            }
        }
    }
}

// TODO: API FOR JOINING SAMPLERS AND TEXTURES

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
    pub unsafe fn bind<'a, T, P>(&'a self, unit: u32, tex: &'a Texture<T, P>, sampler: Option<&Sampler>, gl: &'a Gl) -> BoundTexture<T>
        where T: TextureType,
              P: IntoSampleParameters
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


impl<T, P> Drop for Texture<T, P>
    where T: TextureType,
          P: IntoSampleParameters
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
        impl &Texture<$texture_type:ty> = ($tag_ident:ident, $u_tag_ident:ident, $i_tag_ident:ident);
    )*) => {$(
        unsafe impl<'a, C, P> UniformType for &'a Texture<$texture_type, P>
            where C: ImageFormat,
                  P: IntoSampleParameters
        {
            #[inline]
            fn uniform_tag() -> TypeTag {
                TypeTag::Single(match (C::ATTRIBUTES.scalar_type, C::ATTRIBUTES.scalar_signed) {
                    (GLSLScalarType::Float, _) => TypeBasicTag::$tag_ident,
                    (GLSLScalarType::Int, true) => TypeBasicTag::$i_tag_ident,
                    (GLSLScalarType::Bool, _) |
                    (GLSLScalarType::Int, false) => TypeBasicTag::$u_tag_ident
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
    impl &Texture<targets::SimpleTex<C, D1>> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<targets::SimpleTex<C, D2>> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<targets::SimpleTex<C, D3>> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<targets::CubemapTex<C>> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<targets::RectTex<C>> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<targets::MultisampleTex<C>> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
}

unsafe impl<'a, T, P> UniformType for SampledTexture<'a, T, P>
    where T: TextureType,
          P: IntoSampleParameters,
          &'a Texture<T, P>: UniformType
{
    #[inline]
    fn uniform_tag() -> TypeTag {
        <&'a Texture<T, P> as UniformType>::uniform_tag()
    }
    #[inline]
    unsafe fn upload(&self, loc: GLint, binder: &mut TextureUniformBinder, gl: &Gl) {
        let unit = binder.bind(self.texture, Some(self.sampler), gl);
        gl.Uniform1i(loc, unit as GLint);
    }
}


impl<T> From<TexCreateError<T>> for io::Error
    where T: TextureType,
          TexCreateError<T>: Send + Sync + fmt::Debug
{
    fn from(err: TexCreateError<T>) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl<T: TextureType> Error for TexCreateError<T>
    where TexCreateError<T>: fmt::Debug {}

impl<T> fmt::Display for TexCreateError<T>
    where T: TextureType
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


impl<'a, T, P> Clone for SampledTexture<'a, T, P>
    where T: TextureType,
          P: IntoSampleParameters + 'a
{
    fn clone(&self) -> Self {
        SampledTexture {
            sampler: self.sampler,
            texture: self.texture
        }
    }
}

impl<'a, T, P> Copy for SampledTexture<'a, T, P>
    where T: TextureType,
          P: IntoSampleParameters + 'a {}
