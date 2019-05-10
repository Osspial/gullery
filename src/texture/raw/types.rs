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
use glsl::ScalarType;
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
/// # use gullery::{ContextState, image_format::SRgb, texture::{Texture, types::ArrayTex}};
/// # use glutin::*;
/// # use cgmath_geometry::{D2, rect::DimsBox};
/// #   let el = EventsLoop::new();
/// # let headless = Context::new(
/// #     &el,
/// #     ContextBuilder::new()
/// #         .with_gl_profile(GlProfile::Core)
/// #         .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))),
/// #     false
/// # ).unwrap();
/// # unsafe{ headless.make_current().unwrap() };
/// # let context_state = unsafe{ ContextState::new(|addr| headless.get_proc_address(addr)) };
///
/// const TEXTURE_WIDTH: usize = 256;
/// const TEXTURE_HEIGHT: usize = 128;
///
/// // Create solid-color images for each individual image in the texture array.
/// let red_image = vec![SRgb::new(255, 0, 0); TEXTURE_WIDTH * TEXTURE_HEIGHT];
/// let green_image = vec![SRgb::new(0, 255, 0); TEXTURE_WIDTH * TEXTURE_HEIGHT];
/// let blue_image = vec![SRgb::new(0, 0, 255); TEXTURE_WIDTH * TEXTURE_HEIGHT];
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
/// let array_texture: Texture<D2, ArrayTex<SRgb>> = Texture::with_image(
///     DimsBox::new3(TEXTURE_WIDTH as u32, TEXTURE_HEIGHT as u32, num_images),
///     &combined_image[..],
///     context_state.clone()
/// ).unwrap();
/// ```
///
/// ## GLSL
/// To use this texture type in GLSL, use either a `sampler1DArray` or `sampler2DArray` uniform,
/// for 1D and 2D array texture respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayTex<C>(PhantomData<*const C>)
    where C: ?Sized;

/// Stores six images, each of which representing a face of a cube.
///
/// Refer to the `texture_cubemap` example for full working code using this. Each mipmap in this
/// texture must be uploaded through the [`CubemapImage`] `struct`, which allows you to upload all
/// six faces of the cube at once. Each face must be a perfect square; this constraint is enforced
/// on the type level with the [`DimsSquare`] type.
///
/// ```
/// # extern crate cgmath_geometry;
/// # extern crate glutin;
/// # use gullery::{ContextState, image_format::SRgb, texture::{Texture, DimsSquare, types::{CubemapTex, CubemapImage}}};
/// # use glutin::*;
/// # use std::iter;
/// # use cgmath_geometry::{D2, rect::DimsBox};
/// #   let el = EventsLoop::new();
/// # let headless = Context::new(
/// #     &el,
/// #     ContextBuilder::new()
/// #         .with_gl_profile(GlProfile::Core)
/// #         .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))),
/// #     false
/// # ).unwrap();
/// # unsafe{ headless.make_current().unwrap() };
/// # let context_state = unsafe{ ContextState::new(|addr| headless.get_proc_address(addr)) };
///
/// const TEXTURE_SIDE: usize = 256;
///
/// let bright_red = SRgb::new(255, 0, 0);
/// let bright_blue = SRgb::new(0, 255, 0);
/// let bright_green = SRgb::new(0, 0, 255);
/// let dark_red = SRgb::new(128, 0, 0);
/// let dark_blue = SRgb::new(0, 128, 0);
/// let dark_green = SRgb::new(0, 0, 128);
///
/// let pos_x = vec![bright_red; TEXTURE_SIDE * TEXTURE_SIDE];
/// let pos_y = vec![bright_green; TEXTURE_SIDE * TEXTURE_SIDE];
/// let pos_z = vec![bright_blue; TEXTURE_SIDE * TEXTURE_SIDE];
/// let neg_x = vec![dark_red; TEXTURE_SIDE * TEXTURE_SIDE];
/// let neg_y = vec![dark_green; TEXTURE_SIDE * TEXTURE_SIDE];
/// let neg_z = vec![dark_blue; TEXTURE_SIDE * TEXTURE_SIDE];
///
/// let image = CubemapImage {
///     pos_x: &pos_x,
///     pos_y: &pos_y,
///     pos_z: &pos_z,
///     neg_x: &neg_x,
///     neg_y: &neg_y,
///     neg_z: &neg_z,
/// };
///
/// let cubemap_texture: Texture<D2, CubemapTex<SRgb>> = Texture::with_image(
///     DimsSquare::new(TEXTURE_SIDE as u32),
///     image,
///     context_state.clone()
/// ).unwrap();
/// ```
///
/// ## GLSL
/// To use this in GLSL, use a `samplerCube` uniform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubemapTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;

/// A 2D rectangular texture that contains no mipmaps.
///
/// ## GLSL
/// To use this in GLSL, use a `sampler2DRect` uniform. Note that the texture sampling functions
/// for this type use *non-normalized* texture coordinate, not normalized texture coordinates. This
/// means that sampling a `32x64` texture with `texture({rectTexture}, vec2(0.5, 0.5))` will sample
/// halfway between the `(0, 0)` and `(1, 1)` pixels, instead of halfway into the texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct BufferTex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultisampleTex<C>(PhantomData<*const C>)
    where C: ?Sized + ImageFormat;


/// A single mipmap level on a [`CubemapTex`].
#[derive(Debug, PartialEq, Eq)]
pub struct CubemapImage<'a, I: ImageFormat> {
    pub pos_x: &'a [I],
    pub neg_x: &'a [I],
    pub pos_y: &'a [I],
    pub neg_y: &'a [I],
    pub pos_z: &'a [I],
    pub neg_z: &'a [I],
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


// TRAIT IMPLEMENTATIONS FOR ArrayTex

unsafe impl<D, C> TextureTypeBasicImage<D> for ArrayTex<C>
    where C: ?Sized + ImageFormat,
          D: Dimensionality<u32>,
          ArrayTex<C>: TextureType<D> {}
unsafe impl<C> TextureType<D1> for ArrayTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = ArrayTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D_ARRAY;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size_array(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims_array(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_2d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_2d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
}
unsafe impl<C> TextureType<D2> for ArrayTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ArrayTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size_array(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims_array(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_3d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_3d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
}


// TRAIT IMPLEMENTATIONS FOR BASIC TEXTURES

unsafe impl<D, C> TextureTypeBasicImage<D> for C
    where C: ?Sized + ImageFormat,
          D: Dimensionality<u32>,
          C: TextureType<D> {}
unsafe impl<C> TextureType<D1> for C
    where C: ?Sized + ImageFormat
{
    type MipSelector = u8;
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D1, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_1D;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_1d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_1d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
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
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_2d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_2d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
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
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ImageFormat<ScalarType=C::ScalarType>;

    const BIND_TARGET: GLenum = gl::TEXTURE_3D;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_3d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_3d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
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
    type Samples = ();
    type Format = C;
    type Dims = DimsSquare;

    type Dyn = CubemapTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_CUBE_MAP;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();
        let mip_dims = DimsBox::new2(mip_dims.side, mip_dims.side);

        alloc_image_2d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let sub_dims = DimsBox::new2(sub_dims.side, sub_dims.side);
        sub_image_2d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
}
unsafe impl<C> TextureTypeRenderable<D2> for CubemapTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type DynRenderable = CubemapTex<ImageFormatRenderable<ScalarType=C::ScalarType, FormatType=C::FormatType>>;
}


// TRAIT IMPLEMENTATIONS FOR RectTex

unsafe impl<C> TextureTypeBasicImage<D2> for RectTex<C>
    where C: ?Sized + ImageFormat {}
unsafe impl<C> TextureType<D2> for RectTex<C>
    where C: ?Sized + ImageFormat
{
    type MipSelector = ();
    type Samples = ();
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = RectTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_RECTANGLE;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        mip_level: Self::MipSelector,
        _samples: (),
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        let mip_level = mip_level.to_glint();

        alloc_image_2d(gl, image_bind, mip_dims, mip_level, data_ptr, data_len, Self::Format::FORMAT);
    }
    unsafe fn sub_image(
        gl: &Gl,
        image_bind: GLenum,
        sub_offset: <Self::Dims as Dims>::Offset,
        sub_dims: Self::Dims,
        mip_level: Self::MipSelector,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        sub_image_2d(gl, image_bind, sub_offset, sub_dims, mip_level.to_glint(), data_ptr, data_len, Self::Format::FORMAT);
    }
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


// TRAIT IMPLEMENTATIONS FOR MultisampleTex

unsafe impl<S> TextureType<D2> for MultisampleTex<ImageFormat<ScalarType=S>>
    where S: 'static + ScalarType
{
    type MipSelector = ();
    type Samples = u8;
    type Format = ImageFormat<ScalarType=S>;
    type Dims = DimsBox<D2, u32>;

    type Dyn = MultisampleTex<ImageFormat<ScalarType=S>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        _: &Gl,
        _: GLenum,
        _: Self::Dims,
        _: (),
        _: u8,
        _: *const GLvoid,
        _: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        unreachable!()
    }

    unsafe fn sub_image(
        _: &Gl,
        _: GLenum,
        _: <Self::Dims as Dims>::Offset,
        _: Self::Dims,
        _: Self::MipSelector,
        _: *const GLvoid,
        _: GLsizei
    )
    {
        unreachable!()
    }
}
unsafe impl<C> TextureType<D2> for MultisampleTex<C>
    where C: ?Sized + ImageFormatRenderable
{
    type MipSelector = ();
    type Samples = u8;
    type Format = C;
    type Dims = DimsBox<D2, u32>;

    type Dyn = MultisampleTex<ImageFormat<ScalarType=C::ScalarType>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        _mip_level: (),
        samples: u8,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        assert_eq!(data_ptr, ptr::null());
        assert_eq!(data_len, 0);
        match Self::Format::FORMAT {
            FormatAttributes::Uncompressed{internal_format, ..} => {
                gl.TexImage2DMultisample(
                    image_bind, samples as GLsizei, internal_format as GLenum,
                    mip_dims.width() as GLsizei,
                    mip_dims.height() as GLsizei,
                    0,
                )
            },
            FormatAttributes::Compressed{..} => panic!("Compressed textures cannot be rendered to")
        }
    }

    /// This function should never be called, as the operation is impossible for `MultisampleTex`.
    /// The current implementation panics unconditionally.
    unsafe fn sub_image(
        _: &Gl,
        _: GLenum,
        _: <Self::Dims as Dims>::Offset,
        _: Self::Dims,
        _: Self::MipSelector,
        _: *const GLvoid,
        _: GLsizei
    )
    {
        panic!("MultisampleTex cannot have data uploaded from CPU; must be rendered by GPU. \
                Honestly I'm kinda impressed that you even managed to to call this function. That \
                should never happen in gullery's normal operation, because type-checking should \
                prevent it.");
    }
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
    type Samples = u8;
    type Format = C;
    type Dims = DimsBox<D3, u32>;

    type Dyn = ArrayTex<MultisampleTex<ImageFormat<ScalarType=C::ScalarType>>>;

    const BIND_TARGET: GLenum = gl::TEXTURE_2D_MULTISAMPLE_ARRAY;
    fn max_size(state: &ContextState) -> Self::Dims {Self::Dims::max_size(state)}
    fn mip_dims(dims: Self::Dims, level: Self::MipSelector) -> Self::Dims {dims.mip_dims(level.to_glint())}
    unsafe fn alloc_image(
        gl: &Gl,
        image_bind: GLenum,
        mip_dims: Self::Dims,
        _mip_level: (),
        samples: u8,
        data_ptr: *const GLvoid,
        data_len: GLsizei
    )
        where Self::Format: ConcreteImageFormat
    {
        assert_eq!(data_ptr, ptr::null());
        assert_eq!(data_len, 0);
        match Self::Format::FORMAT {
            FormatAttributes::Uncompressed{internal_format, ..} => {
                gl.TexImage3DMultisample(
                    image_bind, samples as GLsizei, internal_format as GLenum,
                    mip_dims.width() as GLsizei,
                    mip_dims.height() as GLsizei,
                    mip_dims.depth() as GLsizei,
                    0,
                )
            },
            FormatAttributes::Compressed{..} => panic!("Compressed textures cannot be rendered to")
        }
    }

    /// This function should never be called, as the operation is impossible for `MultisampleTex`.
    /// The current implementation panics unconditionally.
    unsafe fn sub_image(
        _: &Gl,
        _: GLenum,
        _: <Self::Dims as Dims>::Offset,
        _: Self::Dims,
        _: Self::MipSelector,
        _: *const GLvoid,
        _: GLsizei
    )
    {
        panic!("MultisampleTex cannot have data uploaded from CPU; must be rendered by GPU.");
    }
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

unsafe fn alloc_image_1d(
    gl: &Gl,
    image_bind: GLenum,
    mip_dims: DimsBox<D1, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{internal_format, pixel_format, pixel_type} => {
            gl.TexImage1D(
                image_bind, mip_level, internal_format as GLint,
                mip_dims.width() as GLsizei,
                0, pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, ..} => {
            gl.CompressedTexImage1D(
                image_bind, mip_level, internal_format,
                mip_dims.width() as GLsizei,
                0, data_len, data_ptr
            )
        }
    }
}

unsafe fn alloc_image_2d(
    gl: &Gl,
    image_bind: GLenum,
    mip_dims: DimsBox<D2, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{internal_format, pixel_format, pixel_type} => {
            gl.TexImage2D(
                image_bind, mip_level, internal_format as GLint,
                mip_dims.width() as GLsizei,
                mip_dims.height() as GLsizei,
                0, pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, ..} => {
            gl.CompressedTexImage2D(
                image_bind, mip_level, internal_format,
                mip_dims.width() as GLsizei,
                mip_dims.height() as GLsizei,
                0, data_len, data_ptr
            )
        }
    }
}

unsafe fn alloc_image_3d(
    gl: &Gl,
    image_bind: GLenum,
    mip_dims: DimsBox<D3, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{internal_format, pixel_format, pixel_type} => {
            gl.TexImage3D(
                image_bind, mip_level, internal_format as GLint,
                mip_dims.width() as GLsizei,
                mip_dims.height() as GLsizei,
                mip_dims.depth() as GLsizei,
                0, pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, ..} => {
            gl.CompressedTexImage3D(
                image_bind, mip_level, internal_format,
                mip_dims.width() as GLsizei,
                mip_dims.height() as GLsizei,
                mip_dims.depth() as GLsizei,
                0, data_len, data_ptr
            )
        }
    }
}

unsafe fn sub_image_1d(
    gl: &Gl,
    image_bind: GLenum,
    sub_offset: Vector1<u32>,
    sub_dims: DimsBox<D1, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{pixel_format, pixel_type, ..} => {
            gl.TexSubImage1D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_dims.width() as GLsizei,
                pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, block_dims} => {
            assert_eq!(sub_offset.x % block_dims.width(), 0);
            assert_eq!(sub_dims.width() % block_dims.width(), 0);

            gl.CompressedTexSubImage1D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_dims.width() as GLsizei,
                internal_format, data_len, data_ptr
            )
        }
    }
}

unsafe fn sub_image_2d(
    gl: &Gl,
    image_bind: GLenum,
    sub_offset: Vector2<u32>,
    sub_dims: DimsBox<D2, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{pixel_format, pixel_type, ..} => {
            gl.TexSubImage2D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_offset.y as GLint,
                sub_dims.width() as GLsizei,
                sub_dims.height() as GLsizei,
                pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, block_dims} => {
            assert_eq!(sub_offset.x % block_dims.width(), 0);
            assert_eq!(sub_offset.y % block_dims.height(), 0);
            assert_eq!(sub_dims.width() % block_dims.width(), 0);
            assert_eq!(sub_dims.height() % block_dims.height(), 0);

            gl.CompressedTexSubImage2D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_offset.y as GLint,
                sub_dims.width() as GLsizei,
                sub_dims.height() as GLsizei,
                internal_format, data_len, data_ptr
            )
        }
    }
}

unsafe fn sub_image_3d(
    gl: &Gl,
    image_bind: GLenum,
    sub_offset: Vector3<u32>,
    sub_dims: DimsBox<D3, u32>,
    mip_level: GLint,
    data_ptr: *const GLvoid,
    data_len: GLsizei,
    format: FormatAttributes,
) {
    match format {
        FormatAttributes::Uncompressed{pixel_format, pixel_type, ..} => {
            gl.TexSubImage3D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_offset.y as GLint,
                sub_offset.z as GLint,
                sub_dims.width() as GLsizei,
                sub_dims.height() as GLsizei,
                sub_dims.depth() as GLsizei,
                pixel_format, pixel_type, data_ptr
            )
        },
        FormatAttributes::Compressed{internal_format, block_dims} => {
            assert_eq!(sub_offset.x % block_dims.width(), 0);
            assert_eq!(sub_offset.y % block_dims.height(), 0);
            assert_eq!(sub_offset.z % block_dims.depth(), 0);
            assert_eq!(sub_dims.width() % block_dims.width(), 0);
            assert_eq!(sub_dims.height() % block_dims.height(), 0);
            assert_eq!(sub_dims.depth() % block_dims.depth(), 0);

            gl.CompressedTexSubImage3D(
                image_bind, mip_level,
                sub_offset.x as GLint,
                sub_offset.y as GLint,
                sub_offset.z as GLint,
                sub_dims.width() as GLsizei,
                sub_dims.height() as GLsizei,
                sub_dims.depth() as GLsizei,
                internal_format, data_len, data_ptr
            )
        }
    }
}
