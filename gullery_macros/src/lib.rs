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

use syn::*;
use quote::{quote, ToTokens};
use proc_macro2::Span;

#[proc_macro_derive(Vertex)]
pub fn derive_vertex(input_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let output = impl_vertex(&derive_input);
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(Uniforms)]
pub fn derive_uniforms(input_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let output = impl_uniforms(&derive_input);
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(Attachments)]
pub fn derive_attachments(input_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let output = impl_attachments(&derive_input);
    proc_macro::TokenStream::from(output)
}

fn impl_vertex(derive_input: &DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        ref ident,
        ref generics,
        ref data,
        ..
    } = *derive_input;

    match *data {
        Data::Enum(..) |
        Data::Union(..) => panic!("Vertex can only be derived on structs"),
        Data::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let idents = idents(variant.fields.iter().cloned());

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate gullery as _gullery;
                    #[automatically_derived]
                    impl #impl_generics _gullery::vertex::Vertex for #ident #ty_generics #where_clause {
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::vertex::VertexMemberRegistry<Group=Self>
                        {
                            #(
                                reg.add_member(stringify!(#idents), |t| unsafe{ &(*t).#idents });
                            )*
                        }
                    }
                };
            }
        }
    }
}

fn impl_uniforms(derive_input: &DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        ref ident,
        ref generics,
        ref data,
        ..
    } = *derive_input;

    match *data {
        Data::Enum(..) |
        Data::Union(..) => panic!("Uniforms can only be derived on structs"),
        Data::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let static_type_generics = static_type_generics(generics);
            let idents = idents(variant.fields.iter().cloned());
            let num_members = variant.fields.iter().len();

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate gullery as _gullery;
                    #[automatically_derived]
                    impl #impl_generics _gullery::uniform::Uniforms for #ident #ty_generics #where_clause {
                        type ULC = [i32; #num_members];
                        type Static = #ident #static_type_generics;
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::uniform::UniformsMemberRegistry<Uniforms=Self>
                        {
                            #(
                                reg.add_member(stringify!(#idents), |t| t.#idents);
                            )*
                        }
                    }
                };
            }
        }
    }
}

fn impl_attachments(derive_input: &DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        ref ident,
        ref generics,
        ref data,
        ..
    } = *derive_input;

    match *data {
        Data::Enum(..) |
        Data::Union(..) => panic!("Attachments can only be derived on structs"),
        Data::Struct(ref variant) => {
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let static_type_generics = static_type_generics(generics);
            let idents = idents(variant.fields.iter().cloned());
            let types = variant.fields.iter().cloned()
                .map(|mut variant| {
                    if let Type::Reference(TypeReference{ref mut lifetime, ..}) = variant.ty {
                        if let Some(_) = *lifetime {
                            *lifetime = None;
                        }
                    }
                    variant.ty
                });
            let types_1 = types.clone();
            let num_members = variant.fields.iter().len();

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate gullery as _gullery;

                    impl #impl_generics #ident #ty_generics #where_clause {
                        /// Check to see that we have at no more than one depth attachment type. If we do,
                        /// we fail to compile.
                        ///
                        /// Thanks to static_assertions crate and rust #49450 for inspiration on how to
                        /// do this.
                        #[allow(dead_code)]
                        fn assert_depth_attachment_number() {
                            union Transmute {
                                from: _gullery::image_format::FormatTypeTag,
                                to: u8
                            }
                            const NUM_DEPTH_ATTACHMENTS: usize = 0
                                #(+ unsafe {
                                    Transmute{ from: <<#types as _gullery::framebuffer::attachments::AttachmentType>::Format as _gullery::image_format::ImageFormatRenderable>::FormatType::FORMAT_TYPE }.to
                                    ==
                                    Transmute{ from: _gullery::image_format::FormatTypeTag::Depth}.to
                                 } as usize)*;
                            let _has_at_least_one_color_attachment = [(); 0 - (NUM_DEPTH_ATTACHMENTS > 1) as usize];
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics _gullery::framebuffer::attachments::Attachments for #ident #ty_generics #where_clause {
                        type AHC = [Option<_gullery::Handle>; #num_members];
                        type Static = #ident #static_type_generics;
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::framebuffer::attachments::AttachmentsMemberRegistry<Attachments=Self>
                        {
                            #(
                                <#types_1 as _gullery::framebuffer::attachments::AttachmentType>::add_to_registry(&mut reg, stringify!(#idents), |t| &t.#idents, Default::default());
                            )*
                        }
                    }
                };
            }
        }
    }
}

fn idents(fields: impl Iterator<Item=Field>) -> impl Iterator<Item=proc_macro2::TokenStream> {
    fields
        .enumerate()
        .map(|(index, variant)| {
            variant.ident.map(|i| i.into_token_stream())
                .unwrap_or_else(|| Index{ index: index as u32, span: Span::call_site()}.into_token_stream())
        })
}

fn static_type_generics(generics: &Generics) -> proc_macro2::TokenStream {
    let static_generics = Generics {
        params: generics.params.iter().cloned().map(|mut p| {
            if let GenericParam::Lifetime(ref mut ld) = p {
                ld.lifetime = Lifetime::new("'static", Span::call_site());
            }
            p
        }).collect(),
        ..generics.clone()
    };
    let (_, type_generics, _) = static_generics.split_for_impl();
    type_generics.into_token_stream()
}
