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

#[proc_macro_derive(TypeGroup)]
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

fn impl_type_group(derive_input: &DeriveInput) -> Tokens {
    let DeriveInput {
        ref ident,
        ref generics,
        ref body,
        ..
    } = *derive_input;

    match *body {
        Body::Enum(..) => panic!("TypeGroup can only be derived on structs"),
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
                    impl #impl_generics _gullery::glsl::TypeGroup for #ident #ty_generics #where_clause {
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::glsl::TyGroupMemberRegistry<Group=Self>
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
        Body::Enum(..) => panic!("TypeGroup can only be derived on structs"),
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

            let dummy_const = Ident::new(format!("_IMPL_TYPE_GROUP_FOR_{}", ident));

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const #dummy_const: () = {
                    extern crate gullery as _gullery;
                    #[automatically_derived]
                    impl #impl_generics _gullery::uniforms::Uniforms for #ident #ty_generics #where_clause {
                        type ULC = [i32; #num_members];
                        type Static = #ident #static_ty_generics;
                        #[inline]
                        fn members<M>(mut reg: M)
                            where M: _gullery::uniforms::UniformsMemberRegistry<Uniforms=Self>
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
