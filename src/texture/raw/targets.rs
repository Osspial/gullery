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
pub struct SimpleTex<C: ?Sized + ImageFormat, D: Dimensionality<u32>>(PhantomData<(*const C, D)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubemapTex<C: ?Sized + ImageFormat>(PhantomData<*const C>);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectTex<C: ?Sized + ImageFormat>(PhantomData<*const C>);
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct BufferTex;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultisampleTex<C: ?Sized + ImageFormat>(PhantomData<*const C>);


unsafe impl<C, D> TextureTypeSingleImage for SimpleTex<C, D>
    where C: ?Sized + ImageFormat,
          D: Dimensionality<u32>,
          SimpleTex<C, D>: TextureType {}
unsafe impl<C> TextureType for SimpleTex<C, D1>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<u32, D1>;

    type Dyn = SimpleTex<ImageFormat<ScalarType=C::ScalarType>, D1>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D;
}
unsafe impl<C> TextureTypeRenderable for SimpleTex<C, D1>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = SimpleTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>, D1>;
}
unsafe impl<C> TextureType for SimpleTex<C, D2>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<u32, D2>;

    type Dyn = SimpleTex<ImageFormat<ScalarType=C::ScalarType>, D2>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D;
}
unsafe impl<C> TextureTypeRenderable for SimpleTex<C, D2>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = SimpleTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>, D2>;
}
unsafe impl<C> TextureType for SimpleTex<C, D3>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<u32, D3>;

    type Dyn = SimpleTex<ImageFormat<ScalarType=C::ScalarType>, D3>;

    const BIND_TARGET: GLenum = gl::TEXTURE_3D;
}
unsafe impl<C> TextureTypeRenderable for SimpleTex<C, D3>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = SimpleTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>, D3>;
}
unsafe impl<C> ArrayTextureType for SimpleTex<C, D1>
    where C: ?Sized + ImageFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_1D_ARRAY;
}
unsafe impl<C> ArrayTextureType for SimpleTex<C, D2>
    where C: ?Sized + ImageFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
}

unsafe impl<C> TextureType for CubemapTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsSquare;

    type Dyn = CubemapTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP;
}
unsafe impl<C> TextureTypeRenderable for CubemapTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = CubemapTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}
// This is an OpenGL 4.0 feature soooo it's not enabled.
// unsafe impl ArrayTextureType for CubemapTex {
//     #[inline]
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP_ARRAY;
// }

unsafe impl<C> TextureTypeSingleImage for RectTex<C>
    where C: ?Sized + ImageFormat {}
unsafe impl<C> TextureType for RectTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<u32, D2>;

    type Dyn = RectTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_RECTANGLE;
}
unsafe impl<C> TextureTypeRenderable for RectTex<C>
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

unsafe impl<C> TextureTypeSingleImage for MultisampleTex<C>
    where C: ?Sized + ImageFormat {}
unsafe impl<C> TextureType for MultisampleTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<u32, D2>;

    type Dyn = MultisampleTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE;
}
unsafe impl<C> TextureTypeRenderable for MultisampleTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = MultisampleTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}
unsafe impl<C> ArrayTextureType for MultisampleTex<C>
    where C: ?Sized + ImageFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE_ARRAY;
}
