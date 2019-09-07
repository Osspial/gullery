// Copyright 2018 Osspial
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::gl::{self, types::*, Gl};

use crate::{
    cgmath::{
        Matrix2, Matrix3, Matrix4, Point1, Point2, Point3, Vector1, Vector2, Vector3, Vector4,
    },
    glsl::{TransparentType, TypeTag},
    image_format::{Red, Rg, Rgb, Rgba},
    texture::{ImageUnits, Sampler, Texture, TextureType},
};
use cgmath_geometry::Dimensionality;
use std::marker::PhantomData;

pub struct TextureUniformBinder<'a> {
    pub(crate) image_units: &'a ImageUnits,
    pub(crate) unit: &'a mut u32,
}

impl<'a> TextureUniformBinder<'a> {
    pub unsafe fn bind<D, T>(
        &mut self,
        tex: &Texture<D, T>,
        sampler: Option<&Sampler>,
        gl: &Gl,
    ) -> u32
    where
        D: Dimensionality<u32>,
        T: ?Sized + TextureType<D>,
    {
        if let Some(sampler) = sampler {
            sampler.upload_parameters();
        } /* else {
              tex.upload_parameters();
          }*/

        let ret = *self.unit;
        self.image_units.bind(*self.unit, tex, sampler, gl);
        *self.unit += 1;
        ret
    }
}

pub unsafe trait UniformType: Copy {
    fn uniform_tag() -> TypeTag;
    unsafe fn upload(&self, loc: GLint, tex_uniform_binder: &mut TextureUniformBinder, gl: &Gl);
}

pub trait Uniforms: Sized + Copy {
    type ULC: UniformLocContainer;
    type Static: 'static + Uniforms<ULC = Self::ULC>;

    fn members<R>(reg: R)
    where
        R: UniformsMemberRegistry<Uniforms = Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, U>(&'a mut usize, PhantomData<U>);
        impl<'a, U: Uniforms> UniformsMemberRegistry for MemberCounter<'a, U> {
            type Uniforms = U;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(Self::Uniforms) -> T)
            where
                T: UniformType,
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
    fn add_member<T: UniformType>(&mut self, name: &str, get_member: fn(Self::Uniforms) -> T);
}

impl Uniforms for () {
    type ULC = [GLint; 0];
    type Static = ();

    #[inline]
    fn members<R>(_: R)
    where
        R: UniformsMemberRegistry<Uniforms = ()>,
    {
    }
}

macro_rules! impl_glsl_type_uniform {
    () => ();
    ($ty_base:ident<$gen0:ty>$(<$gen_more:ty>)+, ($self:ident, $loc:pat, $gl:pat) => $expr:expr, $($rest:tt)*) => {
        impl_glsl_type_uniform!(
            $ty_base<$gen0>, ($self, $loc, $gl) => $expr,
            $ty_base$(<$gen_more>)+, ($self, $loc, $gl) => $expr,
            $($rest)*
        );
    };
    ($ty:ty, ($self:ident, $loc:pat, $gl:pat) => $expr:expr, $($rest:tt)*) => {
        unsafe impl UniformType for $ty {
            #[inline]
            fn uniform_tag() -> TypeTag {
                TypeTag::Single(Self::prim_tag())
            }
            unsafe fn upload(&self, $loc: GLint, _: &mut TextureUniformBinder, $gl: &Gl) {
                let $self = *self;
                $expr
            }
        }

        impl_glsl_type_uniform!($($rest)*);
    };
}
impl_glsl_type_uniform! {
    f32, (f, loc, gl) => gl.Uniform1f(loc, f),
    Vector1<f32>, (v, loc, gl) => gl.Uniform1f(loc, v.x),
    Vector2<f32>, (v, loc, gl) => gl.Uniform2f(loc, v.x, v.y),
    Vector3<f32>, (v, loc, gl) => gl.Uniform3f(loc, v.x, v.y, v.z),
    Vector4<f32>, (v, loc, gl) => gl.Uniform4f(loc, v.x, v.y, v.z, v.w),
    Point1<f32>, (p, loc, gl) => gl.Uniform1f(loc, p.x),
    Point2<f32>, (p, loc, gl) => gl.Uniform2f(loc, p.x, p.y),
    Point3<f32>, (p, loc, gl) => gl.Uniform3f(loc, p.x, p.y, p.z),
    Matrix2<f32>, (m, loc, gl) => gl.UniformMatrix2fv(loc, 1, gl::FALSE, &m.x.x),
    Matrix3<f32>, (m, loc, gl) => gl.UniformMatrix3fv(loc, 1, gl::FALSE, &m.x.x),
    Matrix4<f32>, (m, loc, gl) => gl.UniformMatrix4fv(loc, 1, gl::FALSE, &m.x.x),

    bool, (u, loc, gl) => gl.Uniform1ui(loc, u as u32),
    u32, (u, loc, gl) => gl.Uniform1ui(loc, u),
    Vector1<u32><bool>, (v, loc, gl) => gl.Uniform1ui(loc, v.x as u32),
    Vector2<u32><bool>, (v, loc, gl) => gl.Uniform2ui(loc, v.x as u32, v.y as u32),
    Vector3<u32><bool>, (v, loc, gl) => gl.Uniform3ui(loc, v.x as u32, v.y as u32, v.z as u32),
    Vector4<u32><bool>, (v, loc, gl) => gl.Uniform4ui(loc, v.x as u32, v.y as u32, v.z as u32, v.w as u32),
    Point1<u32><bool>, (p, loc, gl) => gl.Uniform1ui(loc, p.x as u32),
    Point2<u32><bool>, (p, loc, gl) => gl.Uniform2ui(loc, p.x as u32, p.y as u32),
    Point3<u32><bool>, (p, loc, gl) => gl.Uniform3ui(loc, p.x as u32, p.y as u32, p.z as u32),

    i32, (u, loc, gl) => gl.Uniform1i(loc, u),
    Vector1<i32>, (v, loc, gl) => gl.Uniform1i(loc, v.x),
    Vector2<i32>, (v, loc, gl) => gl.Uniform2i(loc, v.x, v.y),
    Vector3<i32>, (v, loc, gl) => gl.Uniform3i(loc, v.x, v.y, v.z),
    Vector4<i32>, (v, loc, gl) => gl.Uniform4i(loc, v.x, v.y, v.z, v.w),
    Point1<i32>, (p, loc, gl) => gl.Uniform1i(loc, p.x),
    Point2<i32>, (p, loc, gl) => gl.Uniform2i(loc, p.x, p.y),
    Point3<i32>, (p, loc, gl) => gl.Uniform3i(loc, p.x, p.y, p.z),

    Rgba<f32>, (c, loc, gl) => gl.Uniform4f(loc, c.r, c.g, c.b, c.a),
    Rgba<u8>, (c, loc, gl) => gl.Uniform4f(loc, c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0),
    Rgba<u16>, (c, loc, gl) => gl.Uniform4f(loc, c.r as f32 / 65536.0, c.g as f32 / 65536.0, c.b as f32 / 65536.0, c.a as f32 / 65536.0),
    Rgba<u32>, (c, loc, gl) => gl.Uniform4f(loc,
        (c.r as f64 / u32::max_value() as f64) as f32,
        (c.g as f64 / u32::max_value() as f64) as f32,
        (c.b as f64 / u32::max_value() as f64) as f32,
        (c.a as f64 / u32::max_value() as f64) as f32,
    ),
    Rgb<f32>, (c, loc, gl) => gl.Uniform3f(loc, c.r, c.g, c.b),
    Rgb<u8>, (c, loc, gl) => gl.Uniform3f(loc, c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0),
    Rgb<u16>, (c, loc, gl) => gl.Uniform3f(loc, c.r as f32 / 65536.0, c.g as f32 / 65536.0, c.b as f32 / 65536.0),
    Rgb<u32>, (c, loc, gl) => gl.Uniform3f(loc,
        (c.r as f64 / u32::max_value() as f64) as f32,
        (c.g as f64 / u32::max_value() as f64) as f32,
        (c.b as f64 / u32::max_value() as f64) as f32,
    ),
    Rg<f32>, (c, loc, gl) => gl.Uniform2f(loc, c.r, c.g),
    Rg<u8>, (c, loc, gl) => gl.Uniform2f(loc, c.r as f32 / 255.0, c.g as f32 / 255.0),
    Rg<u16>, (c, loc, gl) => gl.Uniform2f(loc, c.r as f32 / 65536.0, c.g as f32 / 65536.0),
    Rg<u32>, (c, loc, gl) => gl.Uniform2f(loc,
        (c.r as f64 / u32::max_value() as f64) as f32,
        (c.g as f64 / u32::max_value() as f64) as f32,
    ),
    Red<f32>, (c, loc, gl) => gl.Uniform1f(loc, c.r),
    Red<u8>, (c, loc, gl) => gl.Uniform1f(loc, c.r as f32 / 255.0),
    Red<u16>, (c, loc, gl) => gl.Uniform1f(loc, c.r as f32 / 65536.0),
    Red<u32>, (c, loc, gl) => gl.Uniform1f(loc,
        (c.r as f64 / u32::max_value() as f64) as f32,
    ),
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

// If anybody complains that they need more than 255 uniform fields then idk they can just add
// numbers to this list. However, if you ever need more than 1024 uniform fields (god forbid)
// you're gonna needa add checks because OpenGL defines the minimum value of the maximum number
// of uniform fields as 1024.
impl_ulc_array! {
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32//, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    // 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    // 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    // 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    // 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    // 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
    // 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    // 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
    // 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
    // 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
    // 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
    // 250, 251, 252, 253, 254, 255
}
