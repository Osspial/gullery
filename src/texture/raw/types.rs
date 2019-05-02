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

//! Non-standard texture types.
//!
//! If you want to create a standard texture object, just set the `T` parameter in `Texture` to your
//! desired image format. This module is useful if you want a more complex texture type: in that case
//! set the `T` parameter to `SpecialTex<{ImageFormat}>`

use super::*;
use cgmath_geometry::Dimensionality;

/// Stores multiple logical textures in a single texture object.
///
/// Look at the `texture_array` example for an example of using this. Please note that, when
/// accessing this type of texture, *the dimensionality of types referencing the data contained
/// within is one greater than the dimensionality of the logical texture stored within*. This
/// extra dimension is used to refer to exactly which textures to access in the array.
///
/// For a concrete example of what exactly that means, consider the following:
///
/// ```rust
/// # extern crate cgmath_geometry;
/// # extern crate glutin;
/// # use gullery::{ContextState, image_format::Rgb, texture::{Texture, types::ArrayTex}};
/// # use glutin::*;
/// # use cgmath_geometry::{D2, rect::DimsBox};
/// #   let el = EventsLoop::new();
/// # let headless = Context::new(
/// #     &el,
/// #     ContextBuilder::new()
/// #         .with_gl_profile(GlProfile::Core)
/// #         .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))),
/// #     true
/// # ).unwrap();
/// # unsafe{ headless.make_current().unwrap() };
/// # let context_state = unsafe{ ContextState::new(|addr| headless.get_proc_address(addr)) };
///
/// const TEXTURE_WIDTH: usize = 256;
/// const TEXTURE_HEIGHT: usize = 128;
///
/// // Create solid-color images for each individual image in the texture array.
/// let red_image = [Rgb::new(255, 0, 0); TEXTURE_WIDTH * TEXTURE_HEIGHT];
/// let green_image = [Rgb::new(0, 255, 0); TEXTURE_WIDTH * TEXTURE_HEIGHT];
/// let blue_image = [Rgb::new(0, 0, 255); TEXTURE_WIDTH * TEXTURE_HEIGHT];
///
/// // Combine the above images into a single, contiguous buffer in memory.
/// let mut combined_image = Vec::new();
/// combined_image.extend_from_slice(&red_image);
/// combined_image.extend_from_slice(&green_image);
/// combined_image.extend_from_slice(&blue_image);
///
/// // Calculate the number of images in the buffer, based on each image's size.
/// let num_images = (combined_image.len() / (TEXTURE_WIDTH * TEXTURE_HEIGHT)) as u32;
///
/// // Upload the images to the GPU. Notice how we pass 3D dimensions, instead of 2D dimensions -
/// // the third parameter is the number of textures in the array.
/// let array_texture: Texture<D2, ArrayTex<Rgb>> = Texture::with_images(
///     DimsBox::new3(TEXTURE_WIDTH as u32, TEXTURE_HEIGHT as u32, num_images),
///     Some(&combined_image[..]),
///     context_state.clone()
/// ).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayTex<C>(PhantomData<*const C>)
    where C: ?Sized;

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


#[derive(Debug, PartialEq, Eq)]
pub struct CubemapImage<'a, I: ImageFormat> {
    pub pos_x: &'a [I],
    pub neg_x: &'a [I],
    pub pos_y: &'a [I],
    pub neg_y: &'a [I],
    pub pos_z: &'a [I],
    pub neg_z: &'a [I]
}

impl<'a, I: ImageFormat> Clone for CubemapImage<'a, I> {
    fn clone(&self) -> CubemapImage<'a, I> {
        CubemapImage {
            pos_x: self.pos_x,
            neg_x: self.neg_x,
            pos_y: self.pos_y,
            neg_y: self.neg_y,
            pos_z: self.pos_z,
            neg_z: self.neg_z,
        }
    }
}
impl<'a, I: ImageFormat> Copy for CubemapImage<'a, I> {}


unsafe impl<D, C> TextureTypeSingleImage<D> for ArrayTex<C>
    where C: ?Sized + ImageFormat,
          D: Dimensionality<u32>,
          ArrayTex<C>: TextureType<D> {}
unsafe impl<C> TextureType<D1> for ArrayTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = ArrayTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D_ARRAY;
    const IS_ARRAY: bool = true;
}
unsafe impl<C> TextureType<D2> for ArrayTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ArrayTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
    const IS_ARRAY: bool = true;
}
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
unsafe impl<C> TextureType<D2> for ArrayTex<MultisampleTex<C>>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ArrayTex<MultisampleTex<ImageFormat<ScalarType=C::ScalarType>>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE_ARRAY;
    const IS_ARRAY: bool = true;
}

impl<'a, I> Image<'a, D2, types::CubemapTex<I>> for CubemapImage<'a, I>
    where I: ImageFormat
{
    fn variants<F: FnMut(GLenum, &'a [I])>(self, mut for_each: F) {
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
