use super::*;
pub struct SimpleTex<D: Dims>(PhantomData<D>);

pub struct CubemapTex;

pub struct RectTex;

// pub struct BufferTex;

pub struct MultisampleTex;

impl<D: Dims> Sealed for SimpleTex<D> {}
unsafe impl<C, D> TextureTypeSingleImage<C> for SimpleTex<D>
    where C: ColorFormat, D: Dims {}
unsafe impl<C, D> TextureType<C> for SimpleTex<D>
    where C: ColorFormat, D: Dims
{
    type MipSelector = u8;
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
    type Dims = Dims2d;

    #[inline]
    fn bind_target() -> GLenum {
        gl::TEXTURE_CUBE_MAP
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
unsafe impl<C> TextureTypeSingleImage<C> for RectTex
    where C: ColorFormat {}
unsafe impl<C> TextureType<C> for RectTex
    where C: ColorFormat
{
    type MipSelector = ();
    type Dims = Dims2d;

    #[inline]
    fn bind_target() -> GLenum {
        gl::TEXTURE_RECTANGLE
    }
}

// impl Sealed for BufferTex {}
// unsafe impl TextureType for BufferTex {
//     type MipSelector = ();
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
unsafe impl<C> TextureTypeSingleImage<C> for MultisampleTex
    where C: ColorFormat {}
unsafe impl<C> TextureType<C> for MultisampleTex
    where C: ColorFormat
{
    type MipSelector = ();
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
