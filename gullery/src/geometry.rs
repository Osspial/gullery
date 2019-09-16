use glsl::{GLVec2, GLVec3, NonNormalized, Scalar};
use std::ops::{Add, Sub};

pub trait Dimension<S: Scalar<NonNormalized>>: 'static {
    type Vector: Copy + Add<Output = Self::Vector> + Sub<Output = Self::Vector>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D1 {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D2 {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D3 {}

impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D1 {
    type Vector = S;
}
impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D2 {
    type Vector = GLVec2<S, NonNormalized>;
}
impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D3 {
    type Vector = GLVec3<S, NonNormalized>;
}
