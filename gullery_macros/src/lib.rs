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

#![recursion_limit = "128"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use syn::*;
use quote::Tokens;

#[proc_macro_derive(Vertex)]
pub fn derive_type_group(input_tokens: TokenStream) -> TokenStream {
    let input = input_tokens.to_string();
    let item = syn::parse_derive_input(&input).expect("Attempted derive on non-item");

    let output = impl_type_group(&item).parse().unwrap();
    output
}

#[proc_macro_derive(Uniforms)]
pub fn derive_uniforms(input_tokens: TokenStream) -> TokenStream {
    let input = input_tokens.to_string();
    let item = syn::parse_derive_input(&input).expect("Attempted derive on non-item");

    let output = impl_uniforms(&item).parse().unwrap();
    output
}

#[proc_macro_derive(Attachments)]
pub fn derive_attachments(input_tokens: TokenStream) -> TokenStream {
    let input = input_tokens.to_string();
    let item = syn::parse_derive_input(&input).expect("Attempted derive on non-item");

    let output = impl_attachments(&item).parse().unwrap();
    output
}

fn impl_type_group(derive_input: &DeriveInput) -> Tokens {
    let DeriveInput {
        ref ident,
        ref generics,
        ref body,
        ..
    } = *derive_input;

    match *body {
        Body::Enum(..) => panic!("Vertex can only be derived on structs"),
        Body::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let gen_idents = || variant.fields().iter()
                .cloned()
                .enumerate()
                .map(|(index, mut variant)| {
                    variant.ident = variant.ident.or(Some(Ident::new(index)));
                    variant.ident
                });
            let idents = gen_idents();
            let idents_1 = gen_idents();

            let dummy_const = Ident::new(format!("_IMPL_TYPE_GROUP_FOR_{}", ident));

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const #dummy_const: () = {
                    extern crate gullery as _gullery;
                    #[automatically_derived]
                    impl #impl_generics _gullery::vertex::Vertex for #ident #ty_generics #where_clause {
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::vertex::VertexMemberRegistry<Group=Self>
                        {
                            #(
                                reg.add_member(stringify!(#idents), |t| unsafe{ &(*t).#idents_1 });
                            )*
                        }
                    }
                };
            }
        }
    }
}

fn impl_uniforms(derive_input: &DeriveInput) -> Tokens {
    let DeriveInput {
        ref ident,
        ref generics,
        ref body,
        ..
    } = *derive_input;

    match *body {
        Body::Enum(..) => panic!("Uniforms can only be derived on structs"),
        Body::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let static_generics = Generics {
                lifetimes: generics.lifetimes.iter().cloned().map(|mut ld| {ld.lifetime = Lifetime::new("'static"); ld}).collect(),
                ..generics.clone()
            };
            let (_, static_ty_generics, _) = static_generics.split_for_impl();
            let gen_idents = || variant.fields().iter()
                .cloned()
                .enumerate()
                .map(|(index, mut variant)| {
                    variant.ident = variant.ident.or(Some(Ident::new(index)));
                    variant.ident
                });
            let idents = gen_idents();
            let idents_1 = gen_idents();
            let num_members = variant.fields().len();

            let dummy_const = Ident::new(format!("_IMPL_UNIFORMS_FOR_{}", ident));

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const #dummy_const: () = {
                    extern crate gullery as _gullery;
                    #[automatically_derived]
                    impl #impl_generics _gullery::uniform::Uniforms for #ident #ty_generics #where_clause {
                        type ULC = [i32; #num_members];
                        type Static = #ident #static_ty_generics;
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::uniform::UniformsMemberRegistry<Uniforms=Self>
                        {
                            #(
                                reg.add_member(stringify!(#idents), |t| t.#idents_1);
                            )*
                        }
                    }
                };
            }
        }
    }
}

fn impl_attachments(derive_input: &DeriveInput) -> Tokens {
    let DeriveInput {
        ref ident,
        ref generics,
        ref body,
        ..
    } = *derive_input;

    match *body {
        Body::Enum(..) => panic!("Attachments can only be derived on structs"),
        Body::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let static_generics = Generics {
                lifetimes: generics.lifetimes.iter().cloned().map(|mut ld| {ld.lifetime = Lifetime::new("'static"); ld}).collect(),
                ..generics.clone()
            };
            let (_, static_ty_generics, _) = static_generics.split_for_impl();
            let gen_idents = || variant.fields().iter()
                .cloned()
                .enumerate()
                .map(|(index, mut variant)| {
                    variant.ident = variant.ident.or(Some(Ident::new(index)));
                    variant.ident
                });
            let gen_types = || variant.fields().iter().cloned()
                .map(|mut variant| {
                    if let Ty::Rptr(ref mut lifetime, _) = variant.ty {
                        if let Some(_) = *lifetime {
                            *lifetime = None;
                        }
                    }
                    variant.ty
                });
            let (idents, idents_1) = (gen_idents(), gen_idents());
            let (types, types_1) = (gen_types(), gen_types());
            let num_members = variant.fields().len();

            let dummy_const = Ident::new(format!("_IMPL_ATTACHMENTS_FOR_{}", ident));

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const #dummy_const: () = {
                    extern crate gullery as _gullery;

                    #[automatically_derived]
                    unsafe impl #impl_generics _gullery::framebuffer::attachments::FBOAttachments for #ident #ty_generics #where_clause {}

                    impl #impl_generics #ident #ty_generics #where_clause {
                        /// Check to see that we have at no more than one depth attachment type. If we do,
                        /// we fail to compile.
                        ///
                        /// Thanks to static_assertions crate and rust #49450 for inspiration on how to
                        /// do this.
                        #[allow(dead_code)]
                        fn assert_depth_attachment_number() {
                            union Transmute {
                                from: _gullery::image_format::ImageFormatType,
                                to: u8
                            }
                            const NUM_DEPTH_ATTACHMENTS: usize = 0
                                #(+ unsafe {
                                    Transmute{ from: <<#types as _gullery::framebuffer::attachments::Attachment>::Format as _gullery::image_format::ImageFormat>::FORMAT_TYPE }.to
                                    ==
                                    Transmute{ from: _gullery::image_format::ImageFormatType::Depth}.to
                                 } as usize)*;
                            let _has_at_least_one_color_attachment = [(); 0 - (NUM_DEPTH_ATTACHMENTS > 1) as usize];
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics _gullery::framebuffer::attachments::Attachments for #ident #ty_generics #where_clause {
                        type AHC = [Option<_gullery::Handle>; #num_members];
                        type Static = #ident #static_ty_generics;
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::framebuffer::attachments::AttachmentsMemberRegistry<Attachments=Self>
                        {
                            #(
                                <#types_1 as _gullery::framebuffer::attachments::Attachment>::add_to_registry(&mut reg, stringify!(#idents), |t| &t.#idents_1, Default::default());
                            )*
                        }
                    }
                };
            }
        }
    }
}
