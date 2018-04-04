extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate rand;
extern crate syn;

use std::iter::repeat;
use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn code_gen_builder(input: TokenStream) -> TokenStream {
    let s: String = input.to_string();
    let ast: syn::DeriveInput = syn::parse_derive_input(&s).unwrap();
    let gen: quote::Tokens = impl_builder(&ast);
    gen.parse().unwrap()
}

fn repeat_n(n: usize) -> std::iter::Take<std::iter::Repeat<()>> {
    repeat(()).take(n)
}

fn impl_builder(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let builder_name = syn::Ident::new(format!("{}Builder", name));

    let mod_name = syn::Ident::new(format!("__{}_internal", name).to_lowercase());

    let fields: &Vec<syn::Field> = match &ast.body {
        &syn::Body::Enum(_) => panic!("Cannot derive Builder for enums"),
        &syn::Body::Struct(ref data) => match data {
            &syn::VariantData::Tuple(_) => panic!("Canont derive Builder for Tuple structs"),
            &syn::VariantData::Unit => panic!("Cannot derive Builder for unit structs"),
            &syn::VariantData::Struct(ref fields) => fields,
        },
    };

    let number_of_fields = fields.len();

    let withouts = repeat_n(number_of_fields)
        .map(|_| {
            quote! { #mod_name::Without }
        })
        .collect::<Vec<_>>();

    let field_inits = fields
        .iter()
        .map(|field| {
            //
            let name = field.clone().ident.unwrap();
            quote! { #name: #mod_name::Without }
        })
        .collect::<Vec<_>>();

    let generic_builder_fields = repeat_n(number_of_fields)
        .map(|_| {
            let r = rand::random::<u64>();
            syn::Ident::new(format!("{}Field{}", builder_name, r))
        })
        .collect::<Vec<_>>();

    let builder_fields = fields
        .iter()
        .zip(&generic_builder_fields)
        .map(|(field, ty)| {
            let name = field.clone().ident.unwrap();
            quote! { #name: #ty }
        })
        .collect::<Vec<_>>();

    let done_cons_fields = fields
        .iter()
        .map(|field| {
            let name = field.clone().ident.unwrap();
            quote! { #name: self.#name.item }
        })
        .collect::<Vec<_>>();

    let done_withs = fields
        .iter()
        .map(|field| {
            let ty = field.clone().ty;
            quote! { #mod_name::With<#ty> }
        })
        .collect::<Vec<_>>();

    let fns = fields
        .iter()
        .map(|field| {
            let name = field.clone().ident.unwrap();
            let ty = field.clone().ty;

            let with_withouts = fields
                .iter()
                .enumerate()
                .map(|(idx, other_field)| {
                    if field == other_field {
                        quote! { #mod_name::With<#ty> }
                    } else {
                        let generic_type_name = generic_builder_fields.get(idx).unwrap();
                        quote!{ #generic_type_name }
                    }
                })
                .collect::<Vec<_>>();

            let assigns = fields
                .iter()
                .map(|other_field| {
                    if field == other_field {
                        quote! { #name: #mod_name::With { item: #name.into() } }
                    } else {
                        let name = other_field.clone().ident.unwrap();
                        quote!{ #name: self.#name }
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[inline]
                #[doc(hidden)]
                pub fn #name<T: Into<#ty>>(self, #name: T) -> #builder_name<#(#with_withouts),*> {
                    #builder_name {
                        #(#assigns),*
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let generic_builder_fields = quote! { #(#generic_builder_fields),* };
    let builder_fields = quote! { #(#builder_fields),* };

    quote! {
        mod #mod_name {
            #[derive(Debug, Copy, Clone)]
            #[doc(hidden)]
            pub struct Without;

            #[derive(Debug)]
            #[doc(hidden)]
            pub struct With<T> { pub item: T }
        }

        impl #name {
            #[inline]
            #[doc(hidden)]
            pub fn build() -> #builder_name<#(#withouts),*> {
                #builder_name {
                    #(#field_inits),*
                }
            }
        }

        #[derive(Debug)]
        #[doc(hidden)]
        pub struct #builder_name<#generic_builder_fields> {
            #builder_fields
        }

        impl<#generic_builder_fields> #builder_name<#generic_builder_fields> {
            #(#fns)*
        }

        impl #builder_name<#(#done_withs),*> {
            #[inline]
            #[doc(hidden)]
            pub fn done(self) -> #name {
                #name { #(#done_cons_fields),* }
            }
        }
    }
}
