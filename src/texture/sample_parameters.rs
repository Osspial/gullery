use gl::{self, types::*};
use image_format::Rgba;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Swizzle {
    /// The red channel of the image.
    Red,
    /// The green channel of the image, or `0` if it has no green channel.
    Green,
    /// The blue channel of the image, or `0` if it has no green channel.
    Blue,
    /// The alpha channel of the image, or `1` if it has no green channel.
    Alpha,
    /// Always `0`.
    Zero,
    /// Always `1`.
    One,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMin {
    Nearest,
    Linear,
    NearestMipNearest,
    LinearMipNearest,
    NearestMipLinear,
    LinearMipLinear
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMag {
    Nearest,
    Linear
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapAxis {
    Repeat,
    RepeatMirrored,
    ClampToEdge,
    // ClampToBorder,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureWrap {
    pub s: TextureWrapAxis,
    pub t: TextureWrapAxis,
    pub r: TextureWrapAxis
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleParameters {
    pub filter_min: FilterMin,
    pub filter_mag: FilterMag,
    pub anisotropy_max: f32,
    pub texture_wrap: TextureWrap,
    pub lod_min: f32,
    pub lod_max: f32,
    pub lod_bias: f32,
    // pub border_color: Option<Rgba<f32>>,
    // TODO: GL_TEXTURE_COMPARE_MODE
}

impl Default for FilterMin {
    #[inline(always)]
    fn default() -> FilterMin {FilterMin::Linear}
}

impl Default for FilterMag {
    #[inline(always)]
    fn default() -> FilterMag {FilterMag::Linear}
}

impl Default for TextureWrapAxis {
    #[inline(always)]
    fn default() -> TextureWrapAxis {TextureWrapAxis::Repeat}
}

impl Default for SampleParameters {
    #[inline(always)]
    fn default() -> SampleParameters {
        SampleParameters {
            filter_min: FilterMin::default(),
            filter_mag: FilterMag::default(),
            lod_min: -1000.0,
            lod_max: 1000.0,
            lod_bias: 0.0,
            anisotropy_max: 1.0,
            texture_wrap: TextureWrap::default()
        }
    }
}

impl From<Swizzle> for GLenum {
    #[inline]
    fn from(swizzle: Swizzle) -> GLenum {
        use self::Swizzle::*;
        match swizzle {
            Red => gl::RED,
            Green => gl::GREEN,
            Blue => gl::BLUE,
            Alpha => gl::ALPHA,
            Zero => gl::ZERO,
            One => gl::ONE
        }
    }
}

impl From<FilterMin> for GLenum {
    #[inline]
    fn from(filter: FilterMin) -> GLenum {
        use self::FilterMin::*;
        match filter {
            Nearest => gl::NEAREST,
            Linear => gl::LINEAR,
            NearestMipNearest => gl::NEAREST_MIPMAP_NEAREST,
            LinearMipNearest => gl::LINEAR_MIPMAP_NEAREST,
            NearestMipLinear => gl::NEAREST_MIPMAP_LINEAR,
            LinearMipLinear => gl::LINEAR_MIPMAP_LINEAR
        }
    }
}

impl From<FilterMag> for GLenum {
    #[inline]
    fn from(filter: FilterMag) -> GLenum {
        use self::FilterMag::*;
        match filter {
            Nearest => gl::NEAREST,
            Linear => gl::LINEAR
        }
    }
}

impl From<TextureWrapAxis> for GLenum {
    #[inline]
    fn from(wrap_mode: TextureWrapAxis) -> GLenum {
        use self::TextureWrapAxis::*;
        match wrap_mode {
            Repeat => gl::REPEAT,
            RepeatMirrored => gl::MIRRORED_REPEAT,
            ClampToEdge => gl::CLAMP_TO_EDGE,
            // ClampToBorder = gl::CLAMP_TO_BORDER,
        }
    }
}

impl Default for Rgba<Swizzle> {
    #[inline]
    fn default() -> Rgba<Swizzle> {
        Rgba::new(Swizzle::Red, Swizzle::Green, Swizzle::Blue, Swizzle::Alpha)
    }
}
