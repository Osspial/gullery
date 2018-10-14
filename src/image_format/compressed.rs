use gl;
use image_format::{
    ImageFormat, ConcreteImageFormat, ColorComponents, CompressedFormat, GLFormat,
    ColorFormat, Rgb, Rgba, SRgb, SRgba, Red, Rg
};
use glsl::GLSLFloat;

pub type BC1<S> = DXT1<S>;
pub type BC2<S> = DXT3<S>;
pub type BC3<S> = DXT5<S>;
pub type BC4<S> = RGTC<Red<S>>;
pub type BC5<S> = RGTC<Rg<S>>;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGTC<S: ColorComponents> {
    pub block: [S; 8]
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT1<S: ColorComponents> {
    pub block: [S::Scalar; 8]
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT3<S: ColorComponents> {
    pub block: [S::Scalar; 16]
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT5<S: ColorComponents> {
    pub block: [S::Scalar; 16]
}

impl<S: ColorComponents> RGTC<S> {
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT1<S> {
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT3<S> {
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT5<S> {
    impl_slice_conversions!(S::Scalar);
}


unsafe impl<S: ColorComponents> CompressedFormat for RGTC<S>
    where RGTC<S>: ImageFormat, {}

unsafe impl ImageFormat for RGTC<Red<i8>> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for RGTC<Red<i8>> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SIGNED_RED_RGTC1,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for RGTC<Red<u8>> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for RGTC<Red<u8>> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RED_RGTC1,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for RGTC<Rg<i8>> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for RGTC<Rg<i8>> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SIGNED_RG_RGTC2,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for RGTC<Rg<u8>> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for RGTC<Rg<u8>> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RG_RGTC2,
        pixels_per_block: 4 * 4,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT1<S>
    where DXT1<S>: ImageFormat {}
unsafe impl ImageFormat for DXT1<Rgb> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT1<Rgb> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RGB_S3TC_DXT1_EXT,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for DXT1<Rgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT1<Rgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for DXT1<SRgb> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT1<SRgb> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SRGB_S3TC_DXT1_EXT,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for DXT1<SRgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT1<SRgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT,
        pixels_per_block: 4 * 4,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT3<S>
    where DXT3<S>: ImageFormat {}
unsafe impl ImageFormat for DXT3<Rgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT3<Rgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for DXT3<SRgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT3<SRgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT,
        pixels_per_block: 4 * 4,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT5<S>
    where DXT5<S>: ImageFormat {}
unsafe impl ImageFormat for DXT5<Rgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT5<Rgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
        pixels_per_block: 4 * 4,
    };
}
unsafe impl ImageFormat for DXT5<SRgba> {
    type ScalarType = GLSLFloat;
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for DXT5<SRgba> {
    const FORMAT: GLFormat = GLFormat::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT,
        pixels_per_block: 4 * 4,
    };
}
