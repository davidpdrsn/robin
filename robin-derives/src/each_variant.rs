use quote::Tokens;
use syn::*;

pub fn derive(input: DeriveInput) -> Tokens {
    let name: &Ident = &input.ident;

    let enum_data: DataEnum = match input.data {
        Data::Enum(data) => data,
        _ => panic!("#[derive(EachVariant)] is only defined for enums"),
    };

    let variants = enum_data.variants;
    let variant_names = variants.iter().map(|ref variant| {
        match variant.fields {
            Fields::Unit => {}
            _ => {
                panic!("#[derive(EachVariant)] is only defined on enums where all the variants have no associated values");
            }
        };

        variant.ident
    });

    let push_variants = variant_names
        .map(|variant_name| {
            quote! { acc.push(#name::#variant_name); }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #name {
            #[doc(hidden)]
            pub fn all_variants() -> Vec<Self> {
                let mut acc: Vec<Self> = vec![];
                #(#push_variants);*
                acc
            }
        }
    }
}
