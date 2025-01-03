use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitInt};

#[proc_macro_derive(Error, attributes(error_code))]
pub fn error_code_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    match &ast.data {
        Data::Enum(ref data) => {
            let name = &ast.ident;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

            let variants = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                let mut error_code_attr: Option<u16> = None;
                for attr in variant.clone().attrs {
                    if attr.path().is_ident("error_code") {
                        match attr.parse_args::<LitInt>() {
                            Ok(lit) => error_code_attr = Some(lit.base10_parse::<u16>().unwrap()),
                            Err(err) => panic!("{err}"),
                        };
                    }
                }

                let error_code = match error_code_attr {
                    Some(code) => quote! { #code },
                    None => quote! { 200 }, // Default to 500 if no error_code attribute is found
                };

                match &variant.fields {
                    Fields::Unit => {
                        quote_spanned! { variant.span() =>
                            Self::#variant_name => #error_code,
                        }
                    }
                    Fields::Named(fields) => {
                        // Handle struct variants correctly
                        let field_names: Vec<_> =
                            fields.named.iter().map(|field| &field.ident).collect();
                        quote_spanned! { variant.span() =>
                            Self::#variant_name { #(#field_names),* } => #error_code,
                        }
                    }
                    Fields::Unnamed(_) => {
                        // Handle tuple variants
                        quote_spanned! { variant.span() =>
                            Self::#variant_name(_) => #error_code,
                        }
                    }
                }
            });

            let expanded = quote! {
                impl #impl_generics Error for #name #ty_generics #where_clause {
                    fn error_code(&self) -> u16 {
                        match self {
                            #(#variants)*
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        _ => panic!("Error can only be derived for enums"),
    }
}
