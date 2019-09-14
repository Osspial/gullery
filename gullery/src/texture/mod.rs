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

//! Textures and texture samplers.

#[macro_use]
pub mod sample_parameters;
mod raw;

use crate::gl::{types::*, Gl};

use self::{raw::*, sample_parameters::*};
use crate::{
    image_format::{ConcreteImageFormat, ImageFormat},
    ContextState, GLObject, Handle,
};

use crate::{
    glsl::{ScalarType, TypeTag, TypeTagSingle},
    uniform::{TextureUniformBinder, UniformType},
};

use std::{cell::Cell, error::Error, fmt, io, mem, rc::Rc};

pub use self::raw::{
    types, Dims, DimsSquare, Image, MipSelector, TextureType, TextureTypeBasicImage,
    TextureTypeRenderable,
};

use cgmath_geometry::{Dimensionality, D1, D2, D3};

/// OpenGL Texture object.
// This is repr C in order to guarantee that the `to_dyn` casts work.
#[repr(C)]
pub struct Texture<D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
    raw: RawTexture<D, T>,
    state: Rc<ContextState>,
}

pub struct Sampler {
    pub sample_parameters: SampleParameters,
    old_sample_parameters: Cell<SampleParameters>,

    raw: RawSampler,
    state: Rc<ContextState>,
}

pub struct SampledTexture<'a, D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
    pub sampler: &'a Sampler,
    pub texture: &'a Texture<D, T>,
}

#[derive(Debug, Clone)]
pub enum TextureCreateError<D, T>
where
    D: Dimensionality<u32>,
    T: TextureType<D>,
{
    DimsExceedMax { requested: T::Dims, max: T::Dims },
}

impl<D, T> GLObject for Texture<D, T>
where
    D: Dimensionality<u32>,
    T: TextureType<D>,
{
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

impl GLObject for Sampler {
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

pub(crate) struct ImageUnits(RawImageUnits);
pub(crate) struct BoundTexture<'a, D, T>(RawBoundTexture<'a, D, T>)
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>;

impl<D, T> Texture<D, T>
where
    D: Dimensionality<u32>,
    T: TextureType<D>,
    T::Format: ConcreteImageFormat,
{
    fn check_max_size(dims: T::Dims, state: &ContextState) -> Result<(), TextureCreateError<D, T>> {
        let max_size = T::max_size(&state);
        let (max_width, max_height, max_depth) =
            (max_size.width(), max_size.height(), max_size.depth());
        let (width, height, depth) = (dims.width(), dims.height(), dims.depth());

        if max_width < width || max_height < height || max_depth < depth {
            Err(TextureCreateError::DimsExceedMax {
                requested: dims,
                max: max_size,
            })
        } else {
            Ok(())
        }
    }

    /// Creates a new texture with the given number of mipmaps, without uploading any data to the
    /// GPU.
    ///
    /// The exact data in the texture is unspecified, and shouldn't be relied on to be any specific
    /// value.
    pub fn with_mip_count(
        dims: T::Dims,
        mip_count: u8,
        state: Rc<ContextState>,
    ) -> Result<Texture<D, T>, TextureCreateError<D, T>>
    where
        T: TextureType<D, MipSelector = u8, Samples = ()>,
    {
        Self::check_max_size(dims, &state)?;

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe {
                state
                    .image_units
                    .0
                    .bind_texture_mut(last_unit, &mut raw, &state.gl)
            };
            for level in mip_count.iter_less() {
                bind.alloc_image::<!>(level, (), None);
            }

            if mip_count == 0 {
                panic!("mip_count must be greater than 0");
            }
        }

        Ok(Texture { raw, state })
    }

    /// Creates a new texture with the given images.
    ///
    /// Each image in the `image_mips` iterator is assigned to a mipmap level. As such, each image
    /// must be half the size rounded down on an axis compared to the previous image, with the
    /// minimal size on a given axis being `1`. For example, `[32x8, 16x4, 8x2, 4x1, 2x1, 1x1]`
    /// would be a valid set of image sizes, but `[16x8, 16x4, 8x2, 4x1, 2x1, 1x1]` would not.
    pub fn with_images<'a, I, J>(
        dims: T::Dims,
        image_mips: J,
        state: Rc<ContextState>,
    ) -> Result<Texture<D, T>, TextureCreateError<D, T>>
    where
        T: TextureType<D, MipSelector = u8, Samples = ()>,
        I: Image<'a, D, T>,
        J: IntoIterator<Item = I>,
    {
        Self::check_max_size(dims, &state)?;

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            // We use the last texture unit to make sure that a program never accidentally uses a texture bound
            // during modification. We should probably make sure programs never accidentally use that unit.
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe {
                state
                    .image_units
                    .0
                    .bind_texture_mut(last_unit, &mut raw, &state.gl)
            };

            for (level, image) in image_mips.into_iter().enumerate() {
                bind.alloc_image(level as u8, (), Some(image));
            }

            if bind.raw_tex().num_mips() == 0 {
                panic!("image_mips iterator must contain at least one image");
            }
        }

        Ok(Texture { raw, state })
    }

    pub fn with_image<'a, I>(
        dims: T::Dims,
        image: I,
        state: Rc<ContextState>,
    ) -> Result<Texture<D, T>, TextureCreateError<D, T>>
    where
        T: TextureType<D, Samples = ()>,
        I: Image<'a, D, T>,
    {
        Self::check_max_size(dims, &state)?;

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe {
                state
                    .image_units
                    .0
                    .bind_texture_mut(last_unit, &mut raw, &state.gl)
            };

            bind.alloc_image(T::MipSelector::base(), (), Some(image));
        }

        Ok(Texture { raw, state })
    }

    pub fn with_sample_count<'a>(
        dims: T::Dims,
        samples: u8,
        state: Rc<ContextState>,
    ) -> Result<Texture<D, T>, TextureCreateError<D, T>>
    where
        T: TextureType<D, MipSelector = (), Samples = u8>,
    {
        Self::check_max_size(dims, &state)?;

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            let last_unit = state.image_units.0.num_units() - 1;
            let mut bind = unsafe {
                state
                    .image_units
                    .0
                    .bind_texture_mut(last_unit, &mut raw, &state.gl)
            };

            bind.alloc_image::<!>((), samples, None);
        }

        Ok(Texture { raw, state })
    }

    // You may notice that there's no function for creating a texture with both mipmaps and samples.
    // That's because, to my (Osspial's) knowledge, there are no texture formats in OpenGL Core nor
    // any of its extensions that use both simultaneously, and I don't want to clutter the public
    // API.
    //
    // Full disclaimer: the only reason I say "to my knowledge" is because I can't be assed to read
    // through all the extensions to find a counter-example. If you're hacking on Gullery and have
    // found a texture format that uses both mipmap levels and sample counts, feel free to open a PR
    // with a link to the offending texture format that adds the required functions.

    #[inline]
    pub fn sub_image<'a, I>(
        &mut self,
        mip_level: T::MipSelector,
        offset: <T::Dims as Dims>::Offset,
        sub_dims: T::Dims,
        image: I,
    ) where
        I: Image<'a, D, T>,
    {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe {
            self.state
                .image_units
                .0
                .bind_texture_mut(last_unit, &mut self.raw, &self.state.gl)
        };
        bind.sub_image(mip_level, offset, sub_dims, image);
    }
}

impl<D, T> Texture<D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
    /// The number of mipmap levels the texture has.
    #[inline]
    pub fn num_mips(&self) -> u8 {
        self.raw.num_mips()
    }

    /// The dimensions of the texture.
    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.raw.dims()
    }

    /// Sets the swizzle parameters for when a shader reads from a texture.
    ///
    /// Swizzling lets you change what values a shader reads from a particular texture channel without
    /// actually changing the texture data or the shader. To illustrate, let's imagine you had a 4-bit
    /// 2x2 single-channel image with the following values:
    ///
    /// ```text
    /// 0 5
    /// A F
    /// ```
    ///
    /// No matter how many channels the source image has, shaders will *always* read four-channel RGBA
    /// values from textures. In this case, when a shader reads our single-channel image, the default
    /// behavior is to read the single channel as the red channel, and any other (missing) color
    /// channel as `0`. This would result in a red image getting shown to the user when read from the
    /// shader:
    ///
    /// ```text
    /// rgba rgba
    /// 000F 500F
    /// A00F F00F
    /// ```
    ///
    /// However, single-channel images are often used for non-red images - as alpha masks or grayscale
    /// images, for example. One could modify the shader to re-assign color channels on the GPU, but
    /// in some circumstances it can be easier to set swizzle parameters for the texture, such as when
    /// the shader is also being used with multi-channel textures.
    ///
    /// ## Examples
    ///
    /// These example assume we're using the above single-channel image.
    ///
    /// If you want to read a single-channel image as grayscale, you can set the swizzle parameters to
    /// `(Swizzle::Red, Swizzle::Red, Swizzle::Red, Swizzle::Alpha)`. That will result in the
    /// shader reading out the following values:
    ///
    /// ```text
    /// rgba rgba
    /// 000F 555F
    /// AAAF FFFF
    /// ```
    ///
    /// Alternatively, if you want to read a single-channel image as an alpha mask, you can set the
    /// swizzle parameters to `(Swizzle::One, Swizzle::One, Swizzle::One, Swizzle::Red)`. That
    /// will result in the shader reading out the following values:
    ///
    /// ```text
    /// rgba rgba
    /// FFF0 FFF5
    /// FFFA FFFF
    /// ```
    #[inline]
    pub fn swizzle_read(&mut self, r: Swizzle, g: Swizzle, b: Swizzle, a: Swizzle) {
        let last_unit = self.state.image_units.0.num_units() - 1;
        let mut bind = unsafe {
            self.state
                .image_units
                .0
                .bind_texture_mut(last_unit, &mut self.raw, &self.state.gl)
        };
        bind.swizzle_read(r, g, b, a);
    }

    /// Returns a reference to this texture with the concrete texture type erased.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = Texture::new(DimsBox::new2(512, 512), 1, context_state.clone());
    /// let texture_dyn = &texture as &Texture<D2, dyn ImageFormat<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn as_dyn(&self) -> &Texture<D, T::Dyn> {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::Dyn>>()
        );
        unsafe { &*(self as *const Texture<D, T> as *const Texture<D, T::Dyn>) }
    }

    /// Returns a mutable reference to this texture with the concrete texture type erased.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = /* create texture */;
    /// let texture_dyn = &mut texture as &mut Texture<D2, dyn ImageFormat<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn as_dyn_mut(&mut self) -> &mut Texture<D, T::Dyn> {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::Dyn>>()
        );
        unsafe { &mut *(self as *mut Texture<D, T> as *mut Texture<D, T::Dyn>) }
    }

    /// Returns a texture with the concrete texture type erased.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = /* create texture */;
    /// let texture_dyn = texture as Texture<D2, dyn ImageFormat<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn into_dyn(self) -> Texture<D, T::Dyn> {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::Dyn>>()
        );
        let tex = unsafe { mem::transmute_copy::<Texture<D, T>, Texture<D, T::Dyn>>(&self) };
        mem::forget(self);
        tex
    }

    /// Returns a reference to this texture with the concrete texture type erased, that's usable as
    /// a render target.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = /* create texture */;
    /// let texture_dyn = &texture as &Texture<D2, dyn ImageFormatRenderable<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn as_dyn_renderable(&self) -> &Texture<D, T::DynRenderable>
    where
        T: TextureTypeRenderable<D>,
    {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::DynRenderable>>()
        );
        unsafe { &*(self as *const Texture<D, T> as *const Texture<D, T::DynRenderable>) }
    }

    /// Returns a mutable renderable reference to this texture with the concrete texture type erased,
    /// that's usable as a render target.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = /* create texture */;
    /// let texture_dyn = &mut texture as &Texture<D2, dyn ImageFormatRenderable<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn as_dyn_renderable_mut(&mut self) -> &mut Texture<D, T::DynRenderable>
    where
        T: TextureTypeRenderable<D>,
    {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::DynRenderable>>()
        );
        unsafe { &mut *(self as *mut Texture<D, T> as *mut Texture<D, T::DynRenderable>) }
    }

    /// Returns a renderable texture with the concrete texture type erased, that's usable as a render
    /// target.
    ///
    /// Ideally this function wouldn't be necessary, and you'd be able to do this:
    ///
    /// ```ignore
    /// let texture: Texture<D2, Rgba> = /* create texture */;
    /// let texture_dyn = texture as Texture<D2, dyn ImageFormatRenderable<_>>;
    /// ```
    ///
    /// However, `CoreceUnsized` is not currently stable so we can't.
    #[inline]
    pub fn into_dyn_renderable(self) -> Texture<D, T::DynRenderable>
    where
        T: TextureTypeRenderable<D>,
    {
        assert_eq!(
            mem::size_of::<Texture<D, T>>(),
            mem::size_of::<Texture<D, T::DynRenderable>>()
        );
        let tex =
            unsafe { mem::transmute_copy::<Texture<D, T>, Texture<D, T::DynRenderable>>(&self) };
        mem::forget(self);
        tex
    }
}

impl Sampler {
    pub fn new(context: Rc<ContextState>) -> Sampler {
        Sampler::with_parameters(Default::default(), context)
    }

    pub fn with_parameters(
        sample_parameters: SampleParameters,
        context: Rc<ContextState>,
    ) -> Sampler {
        Sampler {
            raw: RawSampler::new(&context.gl),
            state: context,
            sample_parameters,
            old_sample_parameters: Cell::new(SampleParameters::default()),
        }
    }

    #[inline]
    pub(crate) fn upload_parameters(&self) {
        if self.sample_parameters != self.old_sample_parameters.get() {
            (&self.state.gl, &self.raw)
                .upload_parameters(self.sample_parameters, &self.old_sample_parameters);
        }
    }
}

impl ImageUnits {
    #[inline]
    pub fn new(gl: &Gl) -> ImageUnits {
        ImageUnits(RawImageUnits::new(gl))
    }

    #[inline]
    pub unsafe fn bind<'a, D, T>(
        &'a self,
        unit: u32,
        tex: &'a Texture<D, T>,
        sampler: Option<&Sampler>,
        gl: &'a Gl,
    ) -> BoundTexture<D, T>
    where
        D: Dimensionality<u32>,
        T: ?Sized + TextureType<D>,
    {
        let tex_bind = self.0.bind_texture(unit, &tex.raw, gl);
        match sampler {
            Some(sampler) => self.0.bind_sampler(unit, &sampler.raw, gl),
            None => {
                self.0.unbind_sampler_from_unit(unit, gl);
            }
        }
        BoundTexture(tex_bind)
    }
}

impl<D, T> Drop for Texture<D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state);
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state);
        }
    }
}

macro_rules! texture_type_uniform {
    ($(
        impl &Texture<$d:ty, $texture_type:ty> = ($tag_ident:ident, $u_tag_ident:ident, $i_tag_ident:ident);
    )*) => {$(
        unsafe impl<'a, C> UniformType for &'a Texture<$d, $texture_type>
            where C: ?Sized + ImageFormat,
                  $texture_type: TextureType<$d>
        {
            #[inline]
            fn uniform_tag() -> TypeTag {
                TypeTag::Single(match C::ScalarType::PRIM_TAG {
                    TypeTagSingle::Float => TypeTagSingle::$tag_ident,
                    TypeTagSingle::Int => TypeTagSingle::$i_tag_ident,
                    TypeTagSingle::Bool |
                    TypeTagSingle::UInt => TypeTagSingle::$u_tag_ident,
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

texture_type_uniform! {
    impl &Texture<D1, C> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<D2, C> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<D3, C> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<D1, types::ArrayTex<C>> = (Sampler1DArray, USampler1DArray, ISampler1DArray);
    impl &Texture<D2, types::ArrayTex<C>> = (Sampler2DArray, USampler2DArray, ISampler2DArray);

    impl &Texture<D2, types::CubemapTex<C>> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<D2, types::RectTex<C>> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<D2, types::MultisampleTex<C>> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
    impl &Texture<D2, types::ArrayTex<types::MultisampleTex<C>>> = (Sampler2DMSArray, USampler2DMSArray, ISampler2DMSArray);
}

unsafe impl<'a, D, T> UniformType for SampledTexture<'a, D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
    &'a Texture<D, T>: UniformType,
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

impl<D, T> From<TextureCreateError<D, T>> for io::Error
where
    D: Dimensionality<u32>,
    T: TextureType<D>,
    TextureCreateError<D, T>: Send + Sync + fmt::Debug + fmt::Display,
{
    fn from(err: TextureCreateError<D, T>) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl<D: Dimensionality<u32>, T: TextureType<D>> Error for TextureCreateError<D, T> where
    TextureCreateError<D, T>: fmt::Debug + fmt::Display
{
}

impl<D, T> fmt::Display for TextureCreateError<D, T>
where
    D: Dimensionality<u32>,
    T: TextureType<D>,
    T::Dims: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            TextureCreateError::DimsExceedMax { requested, max } => write!(
                f,
                "requested dimensions {} exceed OpenGL implementation's maximum dimensions {}",
                requested, max,
            ),
        }
    }
}

impl<'a, D, T> Clone for SampledTexture<'a, D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
    fn clone(&self) -> Self {
        SampledTexture {
            sampler: self.sampler,
            texture: self.texture,
        }
    }
}

impl<'a, D, T> Copy for SampledTexture<'a, D, T>
where
    D: Dimensionality<u32>,
    T: ?Sized + TextureType<D>,
{
}
