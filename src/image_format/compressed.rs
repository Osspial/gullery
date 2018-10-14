use gl;
use image_format::{
    ImageFormatType, ImageFormat, ColorComponents, CompressedFormat, GLFormat,
    ImageFormatAttributes, Rgb, Rgba, SRgb, SRgba, Red, Rg
};
use glsl::GLSLScalarType;

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
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SIGNED_RED_RGTC1,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: true,
    };
}
unsafe impl ImageFormat for RGTC<Red<u8>> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RED_RGTC1,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for RGTC<Rg<i8>> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SIGNED_RG_RGTC2,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: true,
    };
}
unsafe impl ImageFormat for RGTC<Rg<u8>> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RG_RGTC2,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT1<S>
    where DXT1<S>: ImageFormat {}
unsafe impl ImageFormat for DXT1<Rgb> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RGB_S3TC_DXT1_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for DXT1<Rgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for DXT1<SRgb> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SRGB_S3TC_DXT1_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for DXT1<SRgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT3<S>
    where DXT3<S>: ImageFormat {}
unsafe impl ImageFormat for DXT3<Rgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for DXT3<SRgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}

unsafe impl<S: ColorComponents> CompressedFormat for DXT5<S>
    where DXT5<S>: ImageFormat {}
unsafe impl ImageFormat for DXT5<Rgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
unsafe impl ImageFormat for DXT5<SRgba> {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Compressed {
            internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT,
            pixels_per_block: 4 * 4,
        },
        format_type: ImageFormatType::Color,
        scalar_type: GLSLScalarType::Float,
        scalar_signed: false,
    };
}
