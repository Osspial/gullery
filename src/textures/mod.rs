mod raw;

use gl::Gl;

use ContextState;
use self::raw::*;
use colors::ColorFormat;

use std::mem;
use std::rc::Rc;

pub use self::raw::{targets, Dims, Dims1d, Dims2d, Dims3d, MipSelector, Image, TextureType, ArrayTextureType};


pub struct Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    raw: RawTexture<C, T>,
    state: Rc<ContextState>
}

pub(crate) struct SamplerUnits(RawSamplerUnits);

impl<C, T> Texture<C, T>
    where C: ColorFormat,
          T: TextureType<C, MipSelector=u8>
{
    pub fn with_images<'a, I, J>(dims: T::Dims, images: J, state: Rc<ContextState>) -> Texture<C, T>
        where I: Image<'a, C, T>,
              J: IntoIterator<Item=I>
    {
        let mut raw = RawTexture::new(dims, &state.gl);

        {
            let active_unit = state.sampler_units.0.active_unit();
            let mut bind = unsafe{ state.sampler_units.0.bind_texture_mut(active_unit, &mut raw, &state.gl) };

            for (level, image) in images.into_iter().enumerate() {
                bind.alloc_image(level as u8, Some(image));
            }

            if bind.raw_tex().num_mips() == 0 {
                bind.alloc_image::<!>(0, None);
            }
        }

        Texture{ raw, state }
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
            let mut bind = unsafe{ state.sampler_units.0.bind_texture_mut(active_unit, &mut raw, &state.gl) };
            for level in mips.iter_less() {
                bind.alloc_image::<!>(level, None);
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
