//! Parameters used to control how a texture gets sampled by shaders.

use crate::{
    gl::{self, types::*},
};

/// Value read from texture, when swizzled.
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

/// The function used to sample from a minified texture.
///
/// Corresponds to `GL_TEXTURE_MIN_FILTER`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMin {
    /// Use nearest-neighbor filtering.
    Nearest,
    /// Weighted average of the four nearest texels.
    Linear,
    /// Choose the mipmap that most closely matches the texture's on-screen size, and perform
    /// [`Nearest`] filtering on that mipmap.
    ///
    /// [`Nearest`]: ./enum.FilterMin.html#variant.Nearest
    NearestMipNearest,
    /// Choose the mipmap that most closely matches the texture's on-screen size, and perform
    /// [`Linear`] filtering on that mipmap.
    ///
    /// [`Linear`]: ./enum.FilterMin.html#variant.Linear
    LinearMipNearest,
    /// **Default value.** Choose the two mipmaps that are closest to the texture's on-screen
    /// size, perform [`Nearest`] filtering on each mipmap, then take the weighted average of the
    /// two mipmaps.
    ///
    /// [`Nearest`]: ./enum.FilterMin.html#variant.Nearest
    NearestMipLinear,
    /// Choose the two mipmaps that are closest to the texture's on-screen size, perform [`Linear`]
    /// filtering on each mipmap, then take the weighted average of the two mipmaps.
    ///
    /// [`Linear`]: ./enum.FilterMin.html#variant.Linear
    LinearMipLinear,
}

/// The function used to sample from a magnified texture.
///
/// Corresponds to `GL_TEXTURE_MAG_FILTER`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMag {
    /// Use nearest-neighbor filtering.
    Nearest,
    /// **Default value.** Weighted average of the four nearest texels.
    Linear,
}

/// The sampling behavior used for coordinates that fall outside of the `0.0..=1.0` range.
///
/// Corresponds to `GL_TEXTURE_WRAP_{axis}`, where `{axis}` is the `s`, `t`, or `r` axis in
/// [`TextureWrap`].
///
/// All example images are based off of this base image:
///
/// ![special thanks to @carols10cents for this plush](https://i.imgur.com/1HEpS2Z.png)
///
/// [`TextureWrap`]: ./struct.TextureWrap.html
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapAxis {
    /// Tile the underyling texture:
    ///
    /// ![](https://i.imgur.com/zZqkS3k.png)
    Repeat,
    /// Tiles the underyling image, mirroring the image on each tile:
    ///
    /// ![](https://i.imgur.com/VUzgqiR.png)
    RepeatMirrored,
    /// Samples the edge pixel of the image:
    ///
    /// ![](https://i.imgur.com/aU56aWT.png)
    ClampToEdge,
    // ClampToBorder,
}

/// The texture's wrapping behavior on each of its axes.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureWrap {
    /// Wrap behavior along the texture's horizontal axis.
    pub s: TextureWrapAxis,
    /// Wrap behavior along the texture's vertical axis.
    pub t: TextureWrapAxis,
    /// Wrap behavior along the texture's depth axis.
    pub r: TextureWrapAxis,
}

/// The texture's LOD sampling parameters.
///
/// # LOD sampling
/// When a texture with multiple LODs gets rendered by the GPU, the GPU needs to calculate which
/// LODs to sample from when outputting the final image. The LOD sample parameter controls that
/// calculation^1. `0.0` samples the 0th LOD, `1.0` samples the 1st LOD, etc. When the sample
/// value is fractional (e.g. `0.4` or `0.75`), the GPU must choose which LODs get sampled to get
/// the final color value. The exact process depends on the `FilterMin` value. The values in this
/// struct modify the final sample value that gets computed.
///
/// <sup>*This is calculated in an implementation-defined way, but generally correlates with the texture's size on screen.</sup>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lod {
    /// A bias that gets added to the LOD sample parameter. This gets applied before the range
    /// bounding from `min` and `max` gets applied.
    pub bias: f32,
    /// The minimum value for the LOD sample parameter.
    pub min: f32,
    /// The maximum value for the LOD sample parameter.
    pub max: f32,
}

/// Collection of parameters that control how a texture gets sampled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleParameters {
    /// The texture's minification filter.
    pub filter_min: FilterMin,
    /// The texture's magnification filter.
    pub filter_mag: FilterMag,
    /// The maximum number of samples used for [anisotropic filtering](https://en.wikipedia.org/wiki/Anisotropic_filtering).
    pub anisotropy_max: f32,
    /// The texture's wrapping behavior on each axis.
    pub texture_wrap: TextureWrap,
    /// The texture's LOD sampling parameters.
    pub lod: Lod,
    // pub border_color: Option<Rgba<f32>>,
    // TODO: GL_TEXTURE_COMPARE_MODE
}

impl Default for FilterMin {
    #[inline(always)]
    fn default() -> FilterMin {
        FilterMin::NearestMipLinear
    }
}

impl Default for FilterMag {
    #[inline(always)]
    fn default() -> FilterMag {
        FilterMag::Linear
    }
}

impl Default for TextureWrapAxis {
    #[inline(always)]
    fn default() -> TextureWrapAxis {
        TextureWrapAxis::Repeat
    }
}

impl Default for SampleParameters {
    #[inline(always)]
    fn default() -> SampleParameters {
        SampleParameters {
            filter_min: FilterMin::default(),
            filter_mag: FilterMag::default(),
            lod: Lod::default(),
            anisotropy_max: 1.0,
            texture_wrap: TextureWrap::default(),
        }
    }
}

impl Default for Lod {
    #[inline(always)]
    fn default() -> Lod {
        Lod {
            min: -1000.0,
            max: 1000.0,
            bias: 0.0,
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
            One => gl::ONE,
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
            LinearMipLinear => gl::LINEAR_MIPMAP_LINEAR,
        }
    }
}

impl From<FilterMag> for GLenum {
    #[inline]
    fn from(filter: FilterMag) -> GLenum {
        use self::FilterMag::*;
        match filter {
            Nearest => gl::NEAREST,
            Linear => gl::LINEAR,
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
