use syn::*;
use quote::Tokens;

pub fn derive(input: DeriveInput) -> Tokens {
    let name: &Ident = &input.ident;

    let enum_data: DataEnum = match input.data {
        Data::Enum(data) => data,
        _ => panic!("#[derive(Jobs)] is only defined for enums"),
    };

    let lookup_job_impl = lookup_job_impl(name, &enum_data);
    let job_impl = job_impl(name, &enum_data);

    quote! {
        #lookup_job_impl
        #job_impl
    }
}

fn lookup_job_impl(name: &Ident, enum_data: &DataEnum) -> Tokens {
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

    quote! {
        pub mod jobs {
            #[inline]
            pub fn lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
                match name.0.as_ref() {
                    #(#lookup_job_match_arms),*
                    _ => None,
                }
            }
        }
    }
}

fn job_impl(name: &Ident, enum_data: &DataEnum) -> Tokens {
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

            'outer: for attr in &variant.attrs {
                for path in attr.path.segments.iter() {
                    if path.ident == Ident::from("perform_with") {
                        let tts = &attr.tts;
                        perform_with = Some(quote! { #tts });
                        break 'outer;
                    }
                }
            }

            let perform_with = match perform_with {
                None => {
                    quote! { #variant_name::perform }
                }
                Some(perform_with) => perform_with,
            };

            quote! {
                #qualified_name => { #perform_with(args.deserialize()?, con) }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl Job for #name {
            #[inline]
            fn name(&self) -> JobName {
                match *self {
                    #(#job_name_match_arms),*
                }
            }

            #[inline]
            fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult {
                match *self {
                    #(#job_perform_match_arms),*
                }
            }
        }
    }
}

fn underscore(s: &str) -> String {
    let mut acc = String::new();

    for (idx, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if idx != 0 {
                acc.push('_');
            }
            acc.push_str(&c.to_lowercase().to_string());
        } else {
            acc.push(c);
        }
    }

    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underscore() {
        assert_eq!(underscore("MyJob"), "my_job".to_string());
        assert_eq!(underscore("myjob"), "myjob".to_string());
        assert_eq!(underscore(""), "".to_string());
        assert_eq!(underscore("Test"), "test".to_string());
        assert_eq!(underscore("test test"), "test test".to_string());
    }
}
