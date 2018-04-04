use syn::*;
use quote::Tokens;

pub fn derive(input: DeriveInput) -> Tokens {
    let name: &Ident = &input.ident;

    match input.data {
        Data::Struct(_) => {}
        _ => panic!("#[derive(Enqueueable)] is only defined for structs"),
    };

    quote! {
        impl Enqueueable for #name {
            fn name(&self) -> JobName {
                JobName::from(stringify!(#name))
            }
        }
    }
}
