use gl::{self, Gl};
use gl::types::*;

use ::seal::Sealed;

use std::mem;
use std::cell::Cell;

pub struct RawTexture<T>
    where T: TextureType<ArrayLayerSelector=!>
{
    handle: GLuint,
    dims: T::Dims,
    mips: u8
}

// pub struct RawTextureArray<T: TextureType>
//     where T: TextureType<ArrayLayerSelector=usize>
// {
//     handle: GLuint,
//     dims: T::Dims,
//     mips: u8,
//     size: usize
// }

pub struct RawTextureTarget<T: TextureType> {
    bound_texture: Cell<GLuint>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimsTagged {
    One(Dims1d),
    Two(Dims2d),
    Three(Dims3d)
}

pub trait Dims: Into<DimsTagged> + Copy + Sealed {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims1d {
    pub width: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims2d {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims3d {
    pub width: u32,
    pub height: u32,
    pub depth: u32
}

impl Dims for Dims1d {}
impl Dims for Dims2d {}
impl Dims for Dims3d {}
impl Sealed for Dims1d {}
impl Sealed for Dims2d {}
impl Sealed for Dims3d {}
impl From<Dims1d> for DimsTagged {
    #[inline]
    fn from(dims: Dims1d) -> DimsTagged {
        DimsTagged::One(dims)
    }
}
impl From<Dims2d> for DimsTagged {
    #[inline]
    fn from(dims: Dims2d) -> DimsTagged {
        DimsTagged::Two(dims)
    }
}
impl From<Dims3d> for DimsTagged {
    #[inline]
    fn from(dims: Dims3d) -> DimsTagged {
        DimsTagged::Three(dims)
    }
}

pub trait MipSelector: Sealed {}
pub trait ImageSelector: Sealed {}
pub trait ArrayLayerSelector: Sealed {}

impl MipSelector for ! {}
impl ImageSelector for ! {}
impl ArrayLayerSelector for ! {}
// Not OpenGL's actual mipmap level type, but if you end up actually using 256 levels then you're
// a madman with damn near infinite VRAM.
impl MipSelector for u8 {}
impl ImageSelector for CubeFace {}
impl Sealed for CubeFace {}
impl ArrayLayerSelector for usize {}


pub unsafe trait TextureType: Sealed {
    type MipSelector: MipSelector;
    type ImageSelector: ImageSelector;
    type ArrayLayerSelector: ArrayLayerSelector;
    type Dims: Dims;

    fn bind_target() -> GLenum;
    fn image_target(_: Self::ImageSelector) -> GLenum;

    fn is_array_texture() -> bool {
        mem::size_of::<Self::ArrayLayerSelector>() != 0
    }
}

impl<T> RawTexture<T>
    where T: TextureType<ArrayLayerSelector=!>
{
    pub fn new(dims: T::Dims, mips: u8, gl: &Gl) -> RawTexture<T> {
        unsafe {
            let mut handle = 0;
            gl.GenTextures(1, &mut handle);
            assert_ne!(0, handle);

            RawTexture{ handle, dims, mips }
        }
    }

    #[inline]
    pub fn dims(&self) -> T::Dims {
        self.dims
    }

    pub fn mips(&self) -> u8 {
        self.mips
    }

    pub fn delete(self, gl: &Gl) {
        unsafe{ gl.DeleteTextures(1, &self.handle) }
    }
}
