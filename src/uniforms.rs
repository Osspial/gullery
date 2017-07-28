use gl::types::*;

use glsl::TypeUniform;

use std::marker::PhantomData;

pub trait Uniforms: Sized + Copy {
    type ULC: UniformLocContainer;
    type Static: 'static + Uniforms<ULC=Self::ULC>;

    fn members<R>(reg: R)
        where R: UniformsMemberRegistry<Uniforms=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, U>(&'a mut usize, PhantomData<U>);
        impl<'a, U: Uniforms> UniformsMemberRegistry for MemberCounter<'a, U> {
            type Uniforms = U;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(Self::Uniforms) -> T)
                where T: TypeUniform
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(MemberCounter::<Self>(&mut num, PhantomData));
        num
    }
}

pub trait UniformLocContainer: AsRef<[GLint]> + AsMut<[GLint]> {
    fn new_zeroed() -> Self;
}

pub trait UniformsMemberRegistry {
    type Uniforms: Uniforms;
    fn add_member<T: TypeUniform>(&mut self, name: &str, get_member: fn(Self::Uniforms) -> T);
}

impl Uniforms for () {
    type ULC = [GLint; 0];
    type Static = ();

    #[inline]
    fn members<R>(_: R)
        where R: UniformsMemberRegistry<Uniforms=()> {}
}

macro_rules! impl_ulc_array {
    ($($len:expr),*) => {$(
        impl UniformLocContainer for [GLint; $len] {
            #[inline]
            fn new_zeroed() -> [GLint; $len] {
                [0; $len]
            }
        }
    )*}
}

impl_ulc_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);
