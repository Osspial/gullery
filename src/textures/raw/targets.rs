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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleTex<C: ColorFormat, D: Dims>(PhantomData<(C, D)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubemapTex<C: ColorFormat>(PhantomData<C>);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectTex<C: ColorFormat>(PhantomData<C>);
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct BufferTex;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultisampleTex<C: ColorFormat>(PhantomData<C>);


unsafe impl<C, D> TextureTypeSingleImage for SimpleTex<C, D>
    where C: ColorFormat,
          D: Dims,
          SimpleTex<C, D>: TextureType {}
unsafe impl<C> TextureType for SimpleTex<C, DimsBox<Point1<u32>>>
    where C: ColorFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<Point1<u32>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D;
}
unsafe impl<C> TextureType for SimpleTex<C, DimsBox<Point2<u32>>>
    where C: ColorFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<Point2<u32>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D;
}
unsafe impl<C> TextureType for SimpleTex<C, DimsBox<Point3<u32>>>
    where C: ColorFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<Point3<u32>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_3D;
}
unsafe impl<C> ArrayTextureType for SimpleTex<C, DimsBox<Point1<u32>>>
    where C: ColorFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_1D_ARRAY;
}
unsafe impl<C> ArrayTextureType for SimpleTex<C, DimsBox<Point2<u32>>>
    where C: ColorFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
}

unsafe impl<C> TextureType for CubemapTex<C>
    where C: ColorFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsSquare;

    const BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP;
}
// This is an OpenGL 4.0 feature soooo it's not enabled.
// unsafe impl ArrayTextureType for CubemapTex {
//     #[inline]
//     const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP_ARRAY;
// }

unsafe impl<C> TextureTypeSingleImage for RectTex<C>
    where C: ColorFormat {}
unsafe impl<C> TextureType for RectTex<C>
    where C: ColorFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<Point2<u32>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_RECTANGLE;
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
    where C: ColorFormat {}
unsafe impl<C> TextureType for MultisampleTex<C>
    where C: ColorFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<Point2<u32>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE;
}
unsafe impl<C> ArrayTextureType for MultisampleTex<C>
    where C: ColorFormat
{
    const ARRAY_BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE_ARRAY;
}
