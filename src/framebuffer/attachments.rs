use colors::ColorFormat;
use renderbuffer::Renderbuffer;
use std::marker::PhantomData;
use GLObject;
use gl::types::*;

pub trait Attachment: GLObject {
    const TARGET_TYPE: AttachmentTargetType;
    const IMAGE_TYPE: AttachmentImageType;
}

pub trait Attachments: Sized {
    type AHC: AttachmentHandleContainer;
    type Static: 'static + Attachments<AHC=Self::AHC>;

    fn members<R>(reg: R)
        where R: AttachmentsMemberRegistry<Attachments=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, A>(&'a mut usize, PhantomData<A>);
        impl<'a, A: Attachments> AttachmentsMemberRegistry for MemberCounter<'a, A> {
            type Attachments = A;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(&Self::Attachments) -> &T)
                where T: Attachment
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(MemberCounter::<Self>(&mut num, PhantomData));
        num
    }
}

pub unsafe trait FBOAttachments: Attachments {}

pub trait AttachmentHandleContainer: AsRef<[GLuint]> + AsMut<[GLuint]> {
    fn new_zeroed() -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentTargetType {
    Renderbuffer,
    // Texture
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentImageType {
    Color,
    // Depth,
    // Stencil,
    // DepthStencil
}

pub trait AttachmentsMemberRegistry {
    type Attachments: Attachments;
    fn add_member<A: Attachment>(
        &mut self,
        name: &str,
        get_member: fn(&Self::Attachments) -> &A
    );
}

macro_rules! impl_attachment_array {
    ($($len:expr),*) => {$(
        impl AttachmentHandleContainer for [GLuint; $len] {
            #[inline]
            fn new_zeroed() -> [GLuint; $len] {
                [0; $len]
            }
        }
    )*}
}

impl_attachment_array!{
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
}

impl Attachments for () {
    type AHC = [GLuint; 0];
    type Static = Self;

    fn members<R>(_reg: R)
        where R: AttachmentsMemberRegistry<Attachments=Self> {}
}

impl<C: ColorFormat> Attachment for Renderbuffer<C> {
    const TARGET_TYPE: AttachmentTargetType = AttachmentTargetType::Renderbuffer;
    const IMAGE_TYPE: AttachmentImageType = AttachmentImageType::Color;
}

impl<'a, A: Attachment> Attachment for &'a mut A {
    const TARGET_TYPE: AttachmentTargetType = A::TARGET_TYPE;
    const IMAGE_TYPE: AttachmentImageType = A::IMAGE_TYPE;
}
