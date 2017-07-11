#![recursion_limit = "128"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use syn::*;
use quote::Tokens;

#[proc_macro_derive(GLSLTyGroup)]
pub fn derive_shader_block(input_tokens: TokenStream) -> TokenStream {
    let input = input_tokens.to_string();
    let item = syn::parse_derive_input(&input).expect("Attempted derive on non-item");

    let output = impl_shader_block(&item).parse().unwrap();
    output
}

fn impl_shader_block(derive_input: &DeriveInput) -> Tokens {
    let DeriveInput {
        ref ident,
        ref generics,
        ref body,
        ..
    } = *derive_input;

    match *body {
        Body::Enum(..) => panic!("GLSLTyGroup can only be derived on structs"),
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

            let dummy_const = Ident::new(format!("_IMPL_SHADER_BLOCK_FOR_{}", ident));

            quote!{
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const #dummy_const: () = {
                    extern crate gl_raii as _gl_raii;
                    #[automatically_derived]
                    impl #impl_generics _gl_raii::GLSLTyGroup for #ident #ty_generics #where_clause  {
                        fn members<M>(mut reg: M)
                            where M: _gl_raii::TyGroupMemberRegistry<Group=Self>
                        {
                            #(
                                reg.add_member(stringify!(#idents), |t| &t.#idents_1);
                            )*
                        }
                    }
                };
            }
        }
    }
}
