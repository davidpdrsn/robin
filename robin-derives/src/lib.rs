extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod each_variant;
mod jobs;

use proc_macro::TokenStream;
use syn::*;
use quote::Tokens;

#[doc(hidden)]
#[proc_macro_derive(EachVariant)]
pub fn derive_each_variant(input: TokenStream) -> TokenStream {
    expand_derive(input, each_variant::derive)
}

#[doc(hidden)]
#[proc_macro_derive(Job, attributes(perform_with))]
pub fn derive_jobs(input: TokenStream) -> TokenStream {
    expand_derive(input, jobs::derive)
}

fn expand_derive<F>(input: TokenStream, f: F) -> TokenStream
where
    F: Fn(DeriveInput) -> Tokens,
{
    let input: DeriveInput = parse(input).unwrap();
    f(input).into()
}
