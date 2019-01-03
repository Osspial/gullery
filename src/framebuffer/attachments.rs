use texture::{Texture, TextureType, MipSelector};
use image_format::{FormatType, ImageFormatRenderable, FormatTypeTag};
use framebuffer::Renderbuffer;
use std::marker::PhantomData;
use cgmath_geometry::Dimensionality;
use {Handle, GLObject};

pub trait AttachmentType: GLObject {
    const TARGET_TYPE: AttachmentTargetType;
    type Format: ?Sized + ImageFormatRenderable;
    type MipSelector: MipSelector;

    fn add_to_registry<R>(
        registry: &mut R,
        name: &str,
        get_member: impl FnOnce(&R::Attachments) -> &Self,
        mip: Self::MipSelector
    )
        where R: AttachmentsMemberRegistry;

    /// Resolve the Attachment to a pointer to the innermost type. For raw types this is a no-op,
    /// but function overloads are used to dereference a `&mut Attachment` to the original value.
    /// Used for input verification in some functions.
    #[doc(hidden)]
    #[inline(always)]
    fn resolve_reference(&self) -> *const () {
        self as *const Self as *const ()
    }
}

/// A collection of `AttachmentTypes`. Should be derived.
///
/// TODO: EXPLAIN MORE.
pub trait Attachments: Sized {
    type AHC: AttachmentHandleContainer;
    type Static: 'static + Attachments<AHC=Self::AHC>;

    fn members<R>(reg: R)
        where R: AttachmentsMemberRegistry<Attachments=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, A>(&'a mut usize, PhantomData<A>);
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for MemberCounter<'a, A> {
            type Attachments = A;
            #[inline(always)]
            fn add_member<At: AttachmentType>(&mut self, _: &str, _: impl FnOnce(&Self::Attachments) -> &At)
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(AMRNSImpl(MemberCounter::<Self>(&mut num, PhantomData)));
        num
    }

    fn color_attachments(&self, for_each: impl FnMut(u8)) {
        struct AttachmentRefMatcher<'a, A: 'a, F: FnMut(u8)> {
            color_index: u8,
            for_each: F,
            _marker: PhantomData<&'a A>
        }
        impl<'a, A: Attachments, F: FnMut(u8)> AttachmentsMemberRegistryNoSpecifics for AttachmentRefMatcher<'a, A, F> {
            type Attachments = A;
            fn add_member<At: AttachmentType>(&mut self, _: &str, _: impl FnOnce(&A) -> &At) {
                let image_type = <At::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE;
                if image_type == FormatTypeTag::Color {
                    (self.for_each)(self.color_index);
                }

                if image_type == FormatTypeTag::Color {
                    self.color_index += 1;
                }
            }
        }

        Self::members(AMRNSImpl(AttachmentRefMatcher {
            color_index: 0,
            for_each,
            _marker: PhantomData
        }));
    }
}

pub unsafe trait FBOAttachments: Attachments {}
pub unsafe trait FramebufferDefaultAttachments: Attachments {}

pub trait AttachmentHandleContainer: AsRef<[Option<Handle>]> + AsMut<[Option<Handle>]> {
    fn new_zeroed() -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentTargetType {
    Renderbuffer,
    Texture
}

pub trait AttachmentsMemberRegistry {
    type Attachments: Attachments;
    fn add_renderbuffer<I: ImageFormatRenderable>(
        &mut self,
        name: &str,
        get_member: impl FnOnce(&Self::Attachments) -> &Renderbuffer<I>
    );
    fn add_texture<D, T>(
        &mut self,
        name: &str,
        get_member: impl FnOnce(&Self::Attachments) -> &Texture<D, T>,
        texture_level: T::MipSelector
    ) where D: Dimensionality<u32>, T: TextureType<D>, T::Format: ImageFormatRenderable;
}

pub(crate) trait AttachmentsMemberRegistryNoSpecifics {
    type Attachments: Attachments;
    fn add_member<A: AttachmentType>(
        &mut self,
        name: &str,
        get_member: impl FnOnce(&Self::Attachments) -> &A
    );
}
pub(crate) struct AMRNSImpl<R: AttachmentsMemberRegistryNoSpecifics>(pub R);
impl<R> AttachmentsMemberRegistry for AMRNSImpl<R>
    where R: AttachmentsMemberRegistryNoSpecifics
{
    type Attachments = <R as AttachmentsMemberRegistryNoSpecifics>::Attachments;
    #[inline]
    fn add_renderbuffer<I>(&mut self, name: &str, get_member: impl FnOnce(&Self::Attachments) -> &Renderbuffer<I>)
        where I: ImageFormatRenderable
    {
        self.0.add_member(name, get_member);
    }
    #[inline]
    fn add_texture<D, T>(&mut self, name: &str, get_member: impl FnOnce(&Self::Attachments) -> &Texture<D, T>, _: T::MipSelector)
        where D: Dimensionality<u32>,
              T: TextureType<D>,
              T::Format: ImageFormatRenderable
    {
        self.0.add_member(name, get_member);
    }
}

macro_rules! impl_attachment_array {
    ($($len:expr),*) => {$(
        impl AttachmentHandleContainer for [Option<Handle>; $len] {
            #[inline]
            fn new_zeroed() -> [Option<Handle>; $len] {
                [None; $len]
            }
        }
    )*}
}

impl_attachment_array!{
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
}

impl Attachments for () {
    type AHC = [Option<Handle>; 0];
    type Static = Self;

    fn members<R>(_reg: R)
        where R: AttachmentsMemberRegistry<Attachments=Self> {}
}
unsafe impl FramebufferDefaultAttachments for () {}

impl<I: ImageFormatRenderable> AttachmentType for Renderbuffer<I> {
    const TARGET_TYPE: AttachmentTargetType = AttachmentTargetType::Renderbuffer;
    type Format = I;
    type MipSelector = ();

    fn add_to_registry<R>(registry: &mut R, name: &str, get_member: impl FnOnce(&R::Attachments) -> &Self, _: ())
        where R: AttachmentsMemberRegistry
    {
        registry.add_renderbuffer(name, |r| get_member(r));
    }
}

impl<D, T> AttachmentType for Texture<D, T>
    where D: Dimensionality<u32>,
          T: TextureType<D>,
          T::Format: ImageFormatRenderable
{
    const TARGET_TYPE: AttachmentTargetType = AttachmentTargetType::Texture;
    type Format = T::Format;
    type MipSelector = T::MipSelector;

    fn add_to_registry<R>(registry: &mut R, name: &str, get_member: impl FnOnce(&R::Attachments) -> &Self, mip: Self::MipSelector)
        where R: AttachmentsMemberRegistry
    {
        registry.add_texture(name, |r| get_member(r), mip);
    }
}

impl<'a, A: 'a + AttachmentType> AttachmentType for &'a mut A {
    const TARGET_TYPE: AttachmentTargetType = A::TARGET_TYPE;
    type Format = A::Format;
    type MipSelector = A::MipSelector;

    fn add_to_registry<R>(registry: &mut R, name: &str, get_member: impl FnOnce(&R::Attachments) -> &Self, mip_selector: A::MipSelector)
        where R: AttachmentsMemberRegistry
    {
        use std::mem;

        A::add_to_registry(
            registry,
            name,
            // We need to retreive a reference to C from our reference to a
            // reference to C. Ideally, we'd use the following line:
            // |r| &**get_member(r)
            //
            // But we transmute because the compiler has trouble with lifetime
            // inference with just a plain call to `&**modify_member(r).
            |r| unsafe{ mem::transmute::<&A, &A>(&**get_member(r)) },
            mip_selector
        );
    }

    fn resolve_reference(&self) -> *const () {
        A::resolve_reference(self)
    }
}
