use gl;
use image_format::{ImageFormatType, ImageFormat, ColorFormat, CompressedFormat, GLFormat, ImageFormatAttributes};
use glsl::GLSLScalarType;

pub type BC4<S> = RGTC_Red<S>;
pub type BC5<S> = RGTC_RG<S>;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGTC_Red<S: Copy> {
    pub red: [S; 8]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGTC_RG<S: Copy> {
    pub red: [S; 8],
    pub green: [S; 8]
}

impl<S: Copy> RGTC_Red<S> {
    impl_slice_conversions!(S);
}

impl<S: Copy> RGTC_RG<S> {
    impl_slice_conversions!(S);
}

unsafe impl<S: Copy> CompressedFormat for RGTC_Red<S>
    where RGTC_Red<S>: ImageFormat {}
unsafe impl<S: Copy> ColorFormat for RGTC_Red<S>
    where RGTC_Red<S>: ImageFormat {}

unsafe impl ImageFormat for RGTC_Red<i8> {
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
unsafe impl ImageFormat for RGTC_Red<u8> {
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


unsafe impl<S: Copy> CompressedFormat for RGTC_RG<S>
    where RGTC_RG<S>: ImageFormat {}
unsafe impl<S: Copy> ColorFormat for RGTC_RG<S>
    where RGTC_RG<S>: ImageFormat {}

unsafe impl ImageFormat for RGTC_RG<i8> {
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
unsafe impl ImageFormat for RGTC_RG<u8> {
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
