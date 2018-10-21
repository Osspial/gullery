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

use super::*;
use cgmath_geometry::Dimensionality;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubemapTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct BufferTex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultisampleTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;


unsafe impl<D, C> TextureTypeSingleImage<D> for C
    where C: ?Sized + ImageFormat,
          D: Dimensionality<u32>,
          C: TextureType<D> {}
unsafe impl<C> TextureType<D1> for C
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<D1, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D;
}
unsafe impl<C> TextureTypeRenderable<D1> for C
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>;
}
unsafe impl<C> TextureType<D2> for C
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D;
}
unsafe impl<C> TextureTypeRenderable<D2> for C
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>;
}
unsafe impl<C> TextureType<D3> for C
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_3D;
}
unsafe impl<C> TextureTypeRenderable<D3> for C
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>;
}
// unsafe impl<C> ArrayTextureType for SimpleTex<D1, C>
//     where C: ?Sized + ImageFormat
// {
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_1D_ARRAY;
// }
// unsafe impl<C> ArrayTextureType for SimpleTex<D2, C>
//     where C: ?Sized + ImageFormat
// {
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
// }

unsafe impl<C> TextureType<D2> for CubemapTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsSquare;

    type Dyn = CubemapTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP;
}
unsafe impl<C> TextureTypeRenderable<D2> for CubemapTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = CubemapTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}
// This is an OpenGL 4.0 feature soooo it's not enabled.
// unsafe impl ArrayTextureType for CubemapTex {
//     #[inline]
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP_ARRAY;
// }

unsafe impl<C> TextureTypeSingleImage<D2> for RectTex<C>
    where C: ?Sized + ImageFormat {}
unsafe impl<C> TextureType<D2> for RectTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = RectTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_RECTANGLE;
}
unsafe impl<C> TextureTypeRenderable<D2> for RectTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = RectTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}

// impl Sealed for BufferTex {}
// unsafe impl TextureType for BufferTex {
//     type MipSelector = ();
//     type Dims = DimsBox<Point1<u32>>;

//     #[inline]
//     fn dims(&self) -> &DimsBox<Point1<u32>> {
//         &self.dims
//     }
//     const BIND_TARGET: GLenum = gl::TEXTURE_BUFFER;
//
// }

unsafe impl<C> TextureTypeSingleImage<D2> for MultisampleTex<C>
    where C: ?Sized + ImageFormat {}
unsafe impl<C> TextureType<D2> for MultisampleTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = MultisampleTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE;
}
unsafe impl<C> TextureTypeRenderable<D2> for MultisampleTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = MultisampleTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}
// unsafe impl<C> ArrayTextureType for MultisampleTex<C>
//     where C: ?Sized + ImageFormat
// {
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE_ARRAY;
// }
