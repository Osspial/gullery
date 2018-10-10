use image_format::{ImageFormatType, ImageFormat, CompressedFormat, ImageFormatAttributes};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BC4<S> {
    pub red: [S; 8]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BC5<S> {
    pub red: [S; 8],
    pub green: [S; 8]
}

// unsafe impl ImageFormat for BC4<i8> {
//     const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
//         internal_format:
//     };
// }
// TODO: UNIMPLEMENT RENDER TO COMPRESSED TEXTURE
