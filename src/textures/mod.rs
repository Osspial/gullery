mod raw;

use gl::Gl;

use ContextState;
use self::raw::*;
use self::raw::error::TextureError;
use colors::ColorFormat;

use glsl::{TypeUniform, TypeTag, TypeBasicTag};
use seal::Sealed;

use std::mem;
use std::rc::Rc;

use num_traits::{PrimInt, Signed};

pub use self::raw::{targets, error, Dims, Dims1D, Dims2D, Dims3D, MipSelector, Image, TextureType, ArrayTextureType};


pub struct Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    raw: RawTexture<C, T>,
    state: Rc<ContextState>
}

pub(crate) struct SamplerUnits(RawSamplerUnits);
pub(crate) struct BoundTexture<'a, C, T>(RawBoundTexture<'a, C, T>)
    where C: ColorFormat,
          T: TextureType<C>;

impl<C, T> Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C, MipSelector=u8>
{
    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Result<Texture<C, T>, TextureError>
        where I: Image<'a, C, T>,
              J: IntoIterator<Item=I>
    {
        let mut raw = RawTexture::new(dims, &state.gl);

        {
            let active_unit = state.sampler_units.0.active_unit();
            let mut bind = unsafe{ state.sampler_units.0.bind_mut(active_unit, &mut raw, &state.gl) };

            for (level, image) in images.into_iter().enumerate() {
                bind.alloc_image(level as u8, Some(image))?;
            }

            if bind.raw_tex().num_mips() == 0 {
                bind.alloc_image::<!>(0, None).expect("Image allocation failed without data. Please file bug report");
            }
        }

        Ok(Texture{ raw, state })
    }
}

impl<C, T> Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn new(dims: T::Dims, mips: T::MipSelector, state: Rc<ContextState>) -> Texture<C, T> {
        let active_unit = state.sampler_units.0.active_unit();

        let mut raw = RawTexture::new(dims, &state.gl);
        {
            let mut bind = unsafe{ state.sampler_units.0.bind_mut(active_unit, &mut raw, &state.gl) };
            for level in mips.iter_less() {
                bind.alloc_image::<!>(level, None).expect("Image allocation failed without data. Please file bug report");
            }
        }

        Texture{ raw, state }
    }

    #[inline]
    pub fn num_mips(&self) -> u8 {
        self.raw.num_mips()
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.raw.dims()
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
    impl &Texture<C, targets::SimpleTex<Dims1D>> = (Sampler1D, USampler1D, ISampler1D);
    impl &Texture<C, targets::SimpleTex<Dims2D>> = (Sampler2D, USampler2D, ISampler2D);
    impl &Texture<C, targets::SimpleTex<Dims3D>> = (Sampler3D, USampler3D, ISampler3D);

    impl &Texture<C, targets::CubemapTex> = (SamplerCube, USamplerCube, ISamplerCube);
    impl &Texture<C, targets::RectTex> = (Sampler2DRect, USampler2DRect, ISampler2DRect);
    impl &Texture<C, targets::MultisampleTex> = (Sampler2DMS, USampler2DMS, ISampler2DMS);
}
