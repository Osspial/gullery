pub mod targets;

use gl::{self, Gl};
use gl::types::*;

use ContextState;
use seal::Sealed;
use colors::ColorFormat;

use std::{mem, ptr, iter};
use std::cell::Cell;
use std::ops::{Deref, Index, Range};
use std::marker::PhantomData;

use cgmath::{Vector1, Vector2, Vector3};

pub struct RawTexture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    handle: GLuint,
    dims: T::Dims,
    num_mips: T::MipSelector,
    _sendsync_optout: PhantomData<*const ()>
}

// pub struct RawTextureArray<T: TextureType>
//     where T: TextureType<ArrayLayerSelector=usize>
// {
//     handle: GLuint,
//     dims: T::Dims,
//     mips: u8,
//     size: usize
// }

pub struct RawSamplerUnits {
    /// The number of sampler units is never going to change, so storing this as Box<[]> means we
    /// don't have to deal with storing the capacity.
    sampler_units: Box<[Cell<GLuint>]>,
    active_unit: Cell<u32>
}

#[repr(C)]
pub struct RawBoundTexture<'a, C, T>
    where C: 'a + ColorFormat,
          T: 'a + TextureType<C>
{
    tex: &'a RawTexture<C, T>,
    gl: &'a Gl
}

#[repr(C)]
pub struct RawBoundTextureMut<'a, C, T>
    where C: 'a + ColorFormat,
          T: 'a + TextureType<C>
{
    tex: &'a mut RawTexture<C, T>,
    gl: &'a Gl
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubeImage<'a, C: ColorFormat> {
    pub pos_x: &'a [C],
    pub neg_x: &'a [C],
    pub pos_y: &'a [C],
    pub neg_y: &'a [C],
    pub pos_z: &'a [C],
    pub neg_z: &'a [C]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimsTag {
    One(Dims1D),
    Two(Dims2D),
    Three(Dims3D)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims1D {
    pub width: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims2D {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimsSquare {
    pub side: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32
}

pub trait Dims: 'static + Into<DimsTag> + Copy + Sealed {
    type Offset: Index<usize, Output=u32>;
    fn num_pixels(self) -> u32;
    fn max_size(state: &ContextState) -> Self;
}

pub unsafe trait TextureType<C: ColorFormat>: 'static + Sealed {
    type MipSelector: MipSelector;
    type Dims: Dims;

    fn bind_target() -> GLenum;
}

pub unsafe trait TextureTypeSingleImage<C: ColorFormat>: TextureType<C> {}

pub unsafe trait ArrayTextureType<C: ColorFormat>: TextureType<C> {
    fn array_bind_target() -> GLenum;
}

pub trait MipSelector: Copy + Sealed {
    type IterLess: Iterator<Item=Self>;

    fn base() -> Self;
    fn to_glint(self) -> GLint;
    fn iter_less(self) -> Self::IterLess;
    fn try_increment(self) -> Self;
}
pub trait Image<'a, C, T>: Copy + Sized + Sealed
    where T: TextureType<C>,
          C: ColorFormat
{
    fn variants<F: FnMut(GLenum, &'a [C])>(self, for_each: F);
    fn variants_static<F: FnMut(GLenum)>(for_each: F);
}


impl<C, T> RawTexture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn new(dims: T::Dims, gl: &Gl) -> RawTexture<C, T> {
        unsafe {
            let mut handle = 0;
            gl.GenTextures(1, &mut handle);
            assert_ne!(0, handle);

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

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.gl.DeleteTextures(1, &self.handle);
            state.sampler_units.0.unbind(self.handle, T::bind_target(), &state.gl);
        }
    }
}

impl RawSamplerUnits {
    pub fn new(gl: &Gl) -> RawSamplerUnits {
        let mut max_tex_units = 0;
        unsafe {
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut max_tex_units);
            gl.PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }
        assert!(0 <= max_tex_units);

        RawSamplerUnits {
            sampler_units: vec![Cell::new(0); max_tex_units as usize].into_boxed_slice(),
            active_unit: Cell::new(0)
        }
    }

    // #[inline]
    // pub fn active_unit(&self) -> u32 {
    //     self.active_unit.get()
    // }

    #[inline]
    pub fn num_units(&self) -> u32 {
        self.sampler_units.len() as u32
    }

    #[inline]
    pub unsafe fn bind<'a, C, T>(&'a self, unit: u32, tex: &'a RawTexture<C, T>, gl: &'a Gl) -> RawBoundTexture<'a, C, T>
        where C: ColorFormat,
              T: 'a + TextureType<C>
    {
        #[inline(never)]
        fn panic_bad_bind(unit: u32, max_unit: u32) -> ! {
            panic!(
                "attempted to bind to unavailable sampler unit {}; highest unit is {}",
                unit,
                max_unit
            );
        }

        let max_unit = self.sampler_units.len() as u32 - 1;

        if max_unit < unit {
            panic_bad_bind(unit, max_unit);
        }

        if unit != self.active_unit.get() {
            self.active_unit.set(unit);
            gl.ActiveTexture(gl::TEXTURE0 + unit);
        }

        let bound_texture = &self.sampler_units[unit as usize];
        if bound_texture.get() != tex.handle {
            bound_texture.set(tex.handle);
            gl.BindTexture(T::bind_target(), tex.handle);
        }

        RawBoundTexture{ tex, gl }
    }

    #[inline]
    pub unsafe fn bind_mut<'a, C, T>(&'a self, unit: u32, tex: &'a mut RawTexture<C, T>, gl: &'a Gl) -> RawBoundTextureMut<'a, C, T>
        where C: ColorFormat,
              T: 'a + TextureType<C>
    {
        self.bind(unit, tex, gl);
        RawBoundTextureMut{ tex, gl }
    }

    unsafe fn unbind(&self, handle: GLuint, target: GLuint, gl: &Gl) {
        for (unit_index, unit) in self.sampler_units.iter().enumerate() {
            if unit.get() == handle {
                gl.ActiveTexture(gl::TEXTURE0 + unit_index as GLuint);
                gl.BindTexture(target, 0);
            }
        }
    }
}


impl<'a, C, T> RawBoundTexture<'a, C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn raw_tex(&self) -> &RawTexture<C, T> {
        &self.tex
    }
}

impl<'a, C, T> RawBoundTextureMut<'a, C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn alloc_image<'b, I>(&mut self, level: T::MipSelector, image: Option<I>)
        where I: Image<'b, C, T>
    {
        unsafe {
            let level_int = level.to_glint();

            if level_int >= self.tex.num_mips() as GLint {
                self.tex.num_mips = level.try_increment();
                self.gl.TexParameteri(T::bind_target(), gl::TEXTURE_MAX_LEVEL, level_int);
            }


            let for_each_variant = |func: fn(&Gl, GLenum, GLint, GLsizei, GLsizei, GLsizei, *const GLvoid)| {
                let (width, height, depth) = self.tex.dims.into().to_tuple();

                let dims_exponent = level_int as u32 + 1;
                let width_gl = width.pow(dims_exponent) as GLsizei;
                let height_gl = height.pow(dims_exponent) as GLsizei;
                let depth_gl = depth.pow(dims_exponent) as GLsizei;

                let num_pixels_expected = (width * height * depth) as usize;

                match image {
                    Some(image_data) => image_data.variants(|image_bind, data| {
                        if data.len() == num_pixels_expected {
                            func(self.gl, image_bind, level_int, width_gl, height_gl, depth_gl, data.as_ptr() as *const GLvoid);
                        } else {
                            panic!("Mismatched image size; expected {} pixels, found {} pixels", num_pixels_expected, data.len());
                        }
                    }),
                    None => I::variants_static(|image_bind| func(self.gl, image_bind, level_int, width_gl, height_gl, depth_gl, ptr::null()))
                }
            };

            match self.tex.dims.into() {
                DimsTag::One(_) => for_each_variant(|gl, image_bind, mip_level, width, _, _, data|
                    gl.TexImage1D(
                        image_bind, mip_level, C::internal_format() as GLint,
                        width,
                        0, C::pixel_format(), C::pixel_type(), data
                    )),
                DimsTag::Two(_) => for_each_variant(|gl, image_bind, mip_level, width, height, _, data|
                    gl.TexImage2D(
                        image_bind, mip_level, C::internal_format() as GLint,
                        width,
                        height,
                        0, C::pixel_format(), C::pixel_type(), data
                    )),
                DimsTag::Three(_) => for_each_variant(|gl, image_bind, mip_level, width, height, depth, data|
                    gl.TexImage3D(
                        image_bind, mip_level, C::internal_format() as GLint,
                        width,
                        height,
                        depth,
                        0, C::pixel_format(), C::pixel_type(), data
                    )),
            }

            assert_eq!(0, self.gl.GetError());
        }
    }

    // pub fn sub_image<'b, I>(
    //     &mut self,
    //     level: T::MipSelector,
    //     image: I,
    //     offset: <T::Dims as Dims>::Offset,
    //     sub_dims: T::Dims
    // ) -> Result<(), TextureError>
    //     where I: Image<'b, C, T>
    // {
    //     use self::DimsTag::*;

    //     let mut result = Ok(());

    //     image.variants(|_, data| {
    //         let num_pixels = data.len() as u32;
    //         if num_pixels != sub_dims.num_pixels() {
    //             result = Err(TextureError::ImageLenMismatch {
    //                 expected: sub_dims.num_pixels() as usize,
    //                 found: num_pixels as usize
    //             });
    //         }
    //     });
    //     result?;

    //     assert!((level.to_glint() as u8) < self.tex.num_mips());
    //     match (self.tex.dims.into(), sub_dims.into()) {
    //         (One(tex_dims), One(sub_dims)) => {
    //             assert!(sub_dims.width <= tex_dims.width);
    //         },
    //         (Two(tex_dims), Two(sub_dims)) => {
    //             assert!(sub_dims.width <= tex_dims.width);
    //             assert!(sub_dims.height <= tex_dims.height);
    //         },
    //         (Three(tex_dims), Three(sub_dims)) => {
    //             assert!(sub_dims.width <= tex_dims.width);
    //             assert!(sub_dims.height <= tex_dims.height);
    //             assert!(sub_dims.depth <= tex_dims.depth);
    //         },
    //         _ => unreachable!()
    //     }

    //     unsafe {
    //         match sub_dims.into() {
    //             One(dims) => image.variants(|image_bind, data| self.gl.TexSubImage1D(
    //                 image_bind, level.to_glint(),
    //                 offset[0] as GLint,
    //                 dims.width as GLsizei,
    //                 C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
    //             )),
    //             Two(dims) => image.variants(|image_bind, data| self.gl.TexSubImage2D(
    //                 image_bind, level.to_glint(),
    //                 offset[0] as GLint,
    //                 offset[1] as GLint,
    //                 dims.width as GLsizei,
    //                 dims.height as GLsizei,
    //                 C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
    //             )),
    //             Three(dims) => image.variants(|image_bind, data| self.gl.TexSubImage3D(
    //                 image_bind, level.to_glint(),
    //                 offset[0] as GLint,
    //                 offset[1] as GLint,
    //                 offset[2] as GLint,
    //                 dims.width as GLsizei,
    //                 dims.height as GLsizei,
    //                 dims.depth as GLsizei,
    //                 C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
    //             ))
    //         }
    //     }

    //     Ok(())
    // }
}


impl<'a, C, T> Deref for RawBoundTextureMut<'a, C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    type Target = RawBoundTexture<'a, C, T>;
    #[inline]
    fn deref(&self) -> &RawBoundTexture<'a, C, T> {
        unsafe{ &*(self as *const _ as *const RawBoundTexture<'a, C, T>) }
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

impl Dims1D {
    #[inline]
    pub fn new(width: u32) -> Dims1D {
        Dims1D{ width }
    }
}
impl Dims2D {
    #[inline]
    pub fn new(width: u32, height: u32) -> Dims2D {
        Dims2D{ width, height }
    }
}
impl DimsSquare {
    #[inline]
    pub fn new(side: u32) -> DimsSquare {
        DimsSquare{ side }
    }
}
impl Dims3D {
    #[inline]
    pub fn new(width: u32, height: u32, depth: u32) -> Dims3D {
        Dims3D{ width, height, depth }
    }
}
impl DimsTag {
    #[inline]
    pub fn to_tuple(self) -> (u32, u32, u32) {
        match self {
            DimsTag::One(dims) => (dims.width, 1, 1),
            DimsTag::Two(dims) => (dims.width, dims.height, 1),
            DimsTag::Three(dims) => (dims.width, dims.height, dims.depth)
        }
    }
}

impl Dims for Dims1D {
    type Offset = Vector1<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width
    }
    #[inline]
    fn max_size(state: &ContextState) -> Dims1D {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            Dims1D::new(size as u32)
        }
    }
}
impl Dims for Dims2D {
    type Offset = Vector2<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width * self.height
    }
    #[inline]
    fn max_size(state: &ContextState) -> Dims2D {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            Dims2D::new(size as u32, size as u32)
        }
    }
}
impl Dims for DimsSquare {
    type Offset = Vector2<u32>;
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
impl Dims for Dims3D {
    type Offset = Vector3<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width * self.height * self.depth
    }
    #[inline]
    fn max_size(state: &ContextState) -> Dims3D {
        unsafe {
            let mut size = 0;
            state.gl.GetIntegerv(gl::MAX_3D_TEXTURE_SIZE, &mut size);
            Dims3D::new(size as u32, size as u32, size as u32)
        }
    }
}
impl From<Dims1D> for DimsTag {
    #[inline]
    fn from(dims: Dims1D) -> DimsTag {
        DimsTag::One(dims)
    }
}
impl From<Dims2D> for DimsTag {
    #[inline]
    fn from(dims: Dims2D) -> DimsTag {
        DimsTag::Two(dims)
    }
}
impl From<DimsSquare> for DimsTag {
    #[inline]
    fn from(dims: DimsSquare) -> DimsTag {
        DimsTag::Two(Dims2D::new(dims.side, dims.side))
    }
}
impl From<Dims3D> for DimsTag {
    #[inline]
    fn from(dims: Dims3D) -> DimsTag {
        DimsTag::Three(dims)
    }
}

impl<'a, C: ColorFormat> Image<'a, C, targets::CubemapTex> for CubeImage<'a, C> {
    fn variants<F: FnMut(GLenum, &'a [C])>(self, mut for_each: F) {
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
impl<'a, C: ColorFormat, T: TextureTypeSingleImage<C>> Image<'a, C, T> for &'a [C] {
    fn variants<F: FnMut(GLenum, &'a [C])>(self, mut for_each: F) {
        for_each(T::bind_target(), self);
    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::bind_target());
   }
}
impl<'a, C: ColorFormat, T: TextureType<C>> Image<'a, C, T> for ! {
    fn variants<F: FnMut(GLenum, &'a [C])>(self, _: F) {    }
    fn variants_static<F: FnMut(GLenum)>(mut for_each: F) {
        for_each(T::bind_target());
   }
}

impl Sealed for Dims1D {}
impl Sealed for Dims2D {}
impl Sealed for DimsSquare {}
impl Sealed for Dims3D {}
impl<'a, C: ColorFormat> Sealed for CubeImage<'a, C> {}
