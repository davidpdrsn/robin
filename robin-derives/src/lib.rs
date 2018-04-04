extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod enqueueable;
mod each_variant;

use proc_macro::TokenStream;
use syn::*;
use quote::Tokens;

macro_rules! derive_impl {
    ($derive_name:ident, $name:ident) => (
        #[doc(hidden)]
        #[proc_macro_derive($derive_name)]
        pub fn $name(input: TokenStream) -> TokenStream {
            expand_derive(input, $name::derive)
        }
    )
}

derive_impl!(Enqueueable, enqueueable);

derive_impl!(EachVariant, each_variant);

fn expand_derive<F>(input: TokenStream, f: F) -> TokenStream
where
    F: Fn(DeriveInput) -> Tokens,
{
    let input: DeriveInput = parse(input).unwrap();
    f(input).into()
}
