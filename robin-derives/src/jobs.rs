use syn::*;
use quote::Tokens;

pub fn derive(input: DeriveInput) -> Tokens {
    let name: &Ident = &input.ident;

    let enum_data: DataEnum = match input.data {
        Data::Enum(data) => data,
        _ => panic!("#[derive(Jobs)] is only defined for enums"),
    };

    let lookup_job_match_arms = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident;
            let qualified_name = quote! { #name::#variant_name };
            quote! {
                stringify!(#qualified_name) => { Some(Box::new(#qualified_name)) }
            }
        })
        .collect::<Vec<_>>();

    let job_name_match_arms = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident;
            let qualified_name = quote! { #name::#variant_name };
            quote! {
                #qualified_name => { JobName::from(stringify!(#qualified_name)) }
            }
        })
        .collect::<Vec<_>>();

    let job_perform_match_arms = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident;
            let qualified_name = quote! { #name::#variant_name };

            let mut perform_with = None;

            for attr in &variant.attrs {
                attr.path.segments.iter().for_each(|path| {
                    if path.ident == Ident::from("perform_with") {
                        let tts = &attr.tts;
                        perform_with = Some(quote! { #tts });
                    };
                })
            }

            match perform_with {
                None => panic!(
                    "#[derive(Jobs)] requires all enum variants to have a `perform_with(function_name)` attribute. `{}::{}` does not.",
                    name,
                    variant_name
                ),
                Some(perform_with) => quote! {
                    #qualified_name => { #perform_with(con, args.deserialize()?) }
                },
            }
        })
        .collect::<Vec<_>>();

    let output = quote! {
        impl #name {
            fn lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
                match name.0.as_ref() {
                    #(#lookup_job_match_arms),*
                    _ => None,
                }
            }
        }

        impl Job for #name {
            fn name(&self) -> JobName {
                match *self {
                    #(#job_name_match_arms),*
                }
            }

            fn perform(&self, con: &WorkerConnection, args: &Args) -> JobResult {
                match *self {
                    #(#job_perform_match_arms),*
                }
            }
        }
    };

    output
}
