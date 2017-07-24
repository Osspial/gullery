use gl::{self, Gl};
use gl::types::*;

use seal::Sealed;
use colors::ColorFormat;

use std::{mem, ptr};
use std::cell::Cell;
use std::ops::Index;
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

pub struct RawBoundTexture<'a, C, T> (PhantomData<&'a RawTexture<C, T>>)
    where C: 'a + ColorFormat,
          T: 'a + TextureType<C>;

pub struct RawBoundTextureMut<'a, C, T>
    where C: 'a + ColorFormat,
          T: 'a + TextureType<C>
{
    tex: &'a RawTexture<C, T>,
    gl: &'a Gl
}

const CUBE_FACE_OFFSET: u32 = 34068;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    PosX = (gl::TEXTURE_CUBE_MAP_POSITIVE_X - CUBE_FACE_OFFSET) as u8,
    NegX = (gl::TEXTURE_CUBE_MAP_NEGATIVE_X - CUBE_FACE_OFFSET) as u8,
    PosY = (gl::TEXTURE_CUBE_MAP_POSITIVE_Y - CUBE_FACE_OFFSET) as u8,
    NegY = (gl::TEXTURE_CUBE_MAP_NEGATIVE_Y - CUBE_FACE_OFFSET) as u8,
    PosZ = (gl::TEXTURE_CUBE_MAP_POSITIVE_Z - CUBE_FACE_OFFSET) as u8,
    NegZ = (gl::TEXTURE_CUBE_MAP_NEGATIVE_Z - CUBE_FACE_OFFSET) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimsTag {
    One(Dims1d),
    Two(Dims2d),
    Three(Dims3d)
}

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

pub trait Dims: Into<DimsTag> + Copy + Sealed {
    type Offset: Index<usize, Output=u32>;
    fn num_pixels(self) -> u32;
}

pub trait MipSelector: Copy + Sealed {
    fn to_glint(self) -> GLint;
}
pub trait ImageSelector: 'static + Copy + Sized + Sealed {
    fn variants() -> &'static [Self];
}


pub unsafe trait TextureType<C: ColorFormat>: Sealed {
    type MipSelector: MipSelector;
    type ImageSelector: ImageSelector;
    type Dims: Dims;

    fn bind_target() -> GLenum;
    #[inline]
    fn image_target(_: Self::ImageSelector) -> GLenum {
        Self::bind_target()
    }
}

pub unsafe trait ArrayTextureType<C: ColorFormat>: TextureType<C> {
    fn array_bind_target() -> GLenum;
}

impl From<CubeFace> for GLenum {
    #[inline]
    fn from(cube_face: CubeFace) -> GLenum {
        let face_discriminant = unsafe{ mem::transmute::<_, u8>(cube_face) };
        face_discriminant as GLenum + CUBE_FACE_OFFSET
    }
}

impl<C, T> RawTexture<C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn new(dims: T::Dims, num_mips: T::MipSelector, gl: &Gl) -> RawTexture<C, T> {
        unsafe {
            let mut handle = 0;
            gl.GenTextures(1, &mut handle);
            assert_ne!(0, handle);

            RawTexture{
                handle,
                dims,
                num_mips,
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

    pub fn delete(self, gl: &Gl) {
        unsafe{ gl.DeleteTextures(1, &self.handle) }
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

    #[inline]
    pub unsafe fn bind_texture<'a, C, T>(&'a self, unit: u32, tex: &'a RawTexture<C, T>, gl: &Gl) -> RawBoundTexture<'a, C, T>
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

        unsafe {
            if unit != self.active_unit.get() {
                self.active_unit.set(unit);
                gl.ActiveTexture(gl::TEXTURE0 + unit);
            }

            let bound_texture = &self.sampler_units[unit as usize];
            if bound_texture.get() != tex.handle {
                bound_texture.set(tex.handle);
                gl.BindTexture(T::bind_target(), tex.handle);
            }
        }

        RawBoundTexture(PhantomData)
    }

    #[inline]
    pub unsafe fn bind_texture_mut<'a, C, T>(&'a self, unit: u32, tex: &'a mut RawTexture<C, T>, gl: &'a Gl) -> RawBoundTextureMut<'a, C, T>
        where C: ColorFormat,
              T: 'a + TextureType<C>
    {
        self.bind_texture(unit, tex, gl);
        RawBoundTextureMut{ tex, gl }
    }
}

impl<'a, C, T> RawBoundTextureMut<'a, C, T>
    where C: ColorFormat,
          T: TextureType<C>
{
    pub fn alloc_image(&mut self) {
        unsafe {
            let exec_tex_image = |tex_image: fn(&Gl, GLenum, GLint, GLsizei, GLsizei, GLsizei)| {
                self.gl.TexParameteri(T::bind_target(), gl::TEXTURE_BASE_LEVEL, 0);
                self.gl.TexParameteri(T::bind_target(), gl::TEXTURE_MAX_LEVEL, self.tex.num_mips() as GLint - 1);

                let (width, height, depth) = match self.tex.dims.into() {
                    DimsTag::One(dims) => (dims.width as GLsizei, 0, 0),
                    DimsTag::Two(dims) => (dims.width as GLsizei, dims.height as GLsizei, 0),
                    DimsTag::Three(dims) => (dims.width as GLsizei, dims.height as GLsizei, dims.depth as GLsizei)
                };

                for level in 0..(self.tex.num_mips() as GLint) {
                    let dims_divisor = 2i32.pow(level as u32 + 1);
                    let (level_width, level_height, level_depth) = (width / dims_divisor, height / dims_divisor, depth / dims_divisor);
                    for variant in T::ImageSelector::variants().iter().map(|v| T::image_target(*v)) {
                        tex_image(self.gl, variant, level, level_width, level_height, level_depth);
                    }
                }
                assert_eq!(0, self.gl.GetError());
            };

            match self.tex.dims.into() {
                DimsTag::One(_) =>
                    exec_tex_image(|gl, variants, level, width, _, _| gl.TexImage1D(
                        variants, level, C::internal_format() as GLint,
                        width,
                        0, C::pixel_format(), C::pixel_type(), ptr::null()
                    )),
                DimsTag::Two(_) =>
                    exec_tex_image(|gl, variants, level, width, height, _| gl.TexImage2D(
                        variants, level, C::internal_format() as GLint,
                        width,
                        height,
                        0, C::pixel_format(), C::pixel_type(), ptr::null()
                    )),
                DimsTag::Three(_) =>
                    exec_tex_image(|gl, variants, level, width, height, depth| gl.TexImage3D(
                        variants, level, C::internal_format() as GLint,
                        width,
                        height,
                        depth,
                        0, C::pixel_format(), C::pixel_type(), ptr::null()
                    ))
            }
        }
    }

    pub fn sub_image(
        &mut self,
        selector: T::ImageSelector,
        level: T::MipSelector,
        offset: <T::Dims as Dims>::Offset,
        sub_dims: T::Dims,
        data: &[C]
    )
    {
        use self::DimsTag::*;
        let num_pixels = (data.len() * C::pixels_per_struct()) as u32;
        if num_pixels != sub_dims.num_pixels() {
            panic!(
                "Mismatched pixel counts; dimensions imply {} pixels, but {} pixels provided",
                sub_dims.num_pixels(),
                num_pixels
            );
        }

        assert!((level.to_glint() as u8) < self.tex.num_mips());
        match (self.tex.dims.into(), sub_dims.into()) {
            (One(tex_dims), One(sub_dims)) => {
                assert!(sub_dims.width <= tex_dims.width);
            },
            (Two(tex_dims), Two(sub_dims)) => {
                assert!(sub_dims.width <= tex_dims.width);
                assert!(sub_dims.height <= tex_dims.height);
            },
            (Three(tex_dims), Three(sub_dims)) => {
                assert!(sub_dims.width <= tex_dims.width);
                assert!(sub_dims.height <= tex_dims.height);
                assert!(sub_dims.depth <= tex_dims.depth);
            },
            _ => unreachable!()
        }

        unsafe {
            match sub_dims.into() {
                One(dims) => self.gl.TexSubImage1D(
                    T::image_target(selector), level.to_glint(),
                    offset[0] as GLint,
                    dims.width as GLsizei,
                    C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
                ),
                Two(dims) => self.gl.TexSubImage2D(
                    T::image_target(selector), level.to_glint(),
                    offset[0] as GLint,
                    offset[1] as GLint,
                    dims.width as GLsizei,
                    dims.height as GLsizei,
                    C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
                ),
                Three(dims) => self.gl.TexSubImage3D(
                    T::image_target(selector), level.to_glint(),
                    offset[0] as GLint,
                    offset[1] as GLint,
                    offset[2] as GLint,
                    dims.width as GLsizei,
                    dims.height as GLsizei,
                    dims.depth as GLsizei,
                    C::pixel_format(), C::pixel_type(), data.as_ptr() as *const GLvoid
                )
            }
        }
    }
}

impl MipSelector for () {
    #[inline]
    fn to_glint(self) -> GLint {
        0
    }
}
impl MipSelector for u8 {
    #[inline]
    fn to_glint(self) -> GLint {
        self as GLint
    }
}

impl ImageSelector for () {
    #[inline]
    fn variants() -> &'static [()] {
        const VARIANTS: &[()] = &[()];
        VARIANTS
    }
}
impl ImageSelector for CubeFace {
    #[inline]
    fn variants() -> &'static [CubeFace] {
        use self::CubeFace::*;
        const VARIANTS: &[CubeFace] = &[PosX, NegX, PosY, NegY, PosZ, NegZ];
        VARIANTS
    }
}


impl Dims for Dims1d {
    type Offset = Vector1<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width
    }
}
impl Dims for Dims2d {
    type Offset = Vector2<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width * self.height
    }
}
impl Dims for Dims3d {
    type Offset = Vector3<u32>;
    #[inline]
    fn num_pixels(self) -> u32 {
        self.width * self.height * self.depth
    }
}
impl From<Dims1d> for DimsTag {
    #[inline]
    fn from(dims: Dims1d) -> DimsTag {
        DimsTag::One(dims)
    }
}
impl From<Dims2d> for DimsTag {
    #[inline]
    fn from(dims: Dims2d) -> DimsTag {
        DimsTag::Two(dims)
    }
}
impl From<Dims3d> for DimsTag {
    #[inline]
    fn from(dims: Dims3d) -> DimsTag {
        DimsTag::Three(dims)
    }
}

impl Sealed for Dims1d {}
impl Sealed for Dims2d {}
impl Sealed for Dims3d {}
impl Sealed for CubeFace {}

pub mod targets {
    use super::*;
    pub struct SimpleTex<D: Dims>(PhantomData<D>);

    pub struct CubemapTex;

    pub struct RectTex;

    // pub struct BufferTex;

    pub struct MultisampleTex;

    impl<D: Dims> Sealed for SimpleTex<D> {}
    unsafe impl<C, D> TextureType<C> for SimpleTex<D>
        where C: ColorFormat, D: Dims
    {
        type MipSelector = u8;
        type ImageSelector = ();
        type Dims = D;

        default fn bind_target() -> GLenum {
            panic!("use specialized version instead.")
        }
    }
    unsafe impl<C> TextureType<C> for SimpleTex<Dims1d>
        where C: ColorFormat
    {
        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_1D
        }
    }
    unsafe impl<C> TextureType<C> for SimpleTex<Dims2d>
        where C: ColorFormat
    {
        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_2D
        }
    }
    unsafe impl<C> TextureType<C> for SimpleTex<Dims3d>
        where C: ColorFormat
    {
        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_3D
        }
    }
    unsafe impl<C> ArrayTextureType<C> for SimpleTex<Dims1d>
        where C: ColorFormat
    {
        #[inline]
        fn array_bind_target() -> GLenum {
            gl::TEXTURE_1D_ARRAY
        }
    }
    unsafe impl<C> ArrayTextureType<C> for SimpleTex<Dims2d>
        where C: ColorFormat
    {
        #[inline]
        fn array_bind_target() -> GLenum {
            gl::TEXTURE_2D_ARRAY
        }
    }

    impl Sealed for CubemapTex {}
    unsafe impl<C> TextureType<C> for CubemapTex
        where C: ColorFormat
    {
        type MipSelector = u8;
        type ImageSelector = CubeFace;
        type Dims = Dims2d;

        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_CUBE_MAP
        }
        #[inline]
        fn image_target(cube_face: CubeFace) -> GLenum {
            cube_face.into()
        }
    }
    // This is an OpenGL 4.0 feature soooo it's not enabled.
    // unsafe impl ArrayTextureType for CubemapTex {
    //     #[inline]
    //     fn array_bind_target() -> GLenum {
    //         gl::TEXTURE_CUBE_MAP_ARRAY
    //     }
    // }

    impl Sealed for RectTex {}
    unsafe impl<C> TextureType<C> for RectTex
        where C: ColorFormat
    {
        type MipSelector = ();
        type ImageSelector = ();
        type Dims = Dims2d;

        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_RECTANGLE
        }
    }

    // impl Sealed for BufferTex {}
    // unsafe impl TextureType for BufferTex {
    //     type MipSelector = ();
    //     type ImageSelector = ();
    //     type Dims = Dims1d;

    //     #[inline]
    //     fn dims(&self) -> &Dims1d {
    //         &self.dims
    //     }
    //     #[inline]
    //     fn bind_target() -> GLenum {
    //         gl::TEXTURE_BUFFER
    //     }
    // }

    impl Sealed for MultisampleTex {}
    unsafe impl<C> TextureType<C> for MultisampleTex
        where C: ColorFormat
    {
        type MipSelector = ();
        type ImageSelector = ();
        type Dims = Dims2d;

        #[inline]
        fn bind_target() -> GLenum {
            gl::TEXTURE_2D_MULTISAMPLE
        }
    }
    unsafe impl<C> ArrayTextureType<C> for MultisampleTex
        where C: ColorFormat
    {
        #[inline]
        fn array_bind_target() -> GLenum {
            gl::TEXTURE_2D_MULTISAMPLE_ARRAY
        }
    }
}
