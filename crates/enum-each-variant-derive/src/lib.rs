//! Derive method that returns each variant of an enum
//!
//! # Sample usage
//!
//! ```rust
//! #[macro_use]
//! extern crate enum_each_variant_derive;
//!
//! # fn main() {
//! #[derive(EachVariant, Eq, PartialEq, Debug)]
//! enum Thing {
//!     One,
//!     Two,
//!     Three,
//!     Four,
//! }
//!
//! let all: Vec<Thing> = Thing::all_variants();
//!
//! assert_eq!(all, vec![Thing::One, Thing::Two, Thing::Three, Thing::Four]);
//! # }
//! ```
//!
//! # Gotcha
//!
//! Only works on enums where no variants have associated values. So we wouldn't be able to use it
//! for this enum:
//!
//! ```rust
//! # fn main() {
//! enum TrainStatus {
//!     OnTime,
//!     DelayedBy(std::time::Duration),
//! }
//! # }
//! ```

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::*;

#[doc(hidden)]
#[proc_macro_derive(EachVariant)]
pub fn each_variant(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse(input).unwrap();
    let expanded = impl_enum_each(input);
    expanded.into()
}

fn impl_enum_each(ast: DeriveInput) -> quote::Tokens {
    let name: &Ident = &ast.ident;

    let enum_data: DataEnum = match ast.data {
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
            pub fn all_variants() -> Vec<Self> {
                let mut acc: Vec<Self> = vec![];
                #(#push_variants);*
                acc
            }
        }
    }
}
