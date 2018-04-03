extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::*;

#[doc(hidden)]
#[proc_macro_derive(Enqueueable)]
pub fn derive_enqueueable(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse(input).unwrap();

    let name: &Ident = &input.ident;

    match input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => panic!("#[derive(Enqueueable)] is only defined for structs"),
        Data::Union(_) => panic!("#[derive(Enqueueable)] is only defined for structs"),
    };

    let expanded = quote! {
        impl Enqueueable for #name {
            fn name(&self) -> JobName {
                JobName::from(stringify!(#name))
            }
        }
    };

    expanded.into()
}
