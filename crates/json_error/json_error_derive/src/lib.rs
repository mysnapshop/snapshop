use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitInt};

#[proc_macro_derive(ErrorCode, attributes(error_code))]
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
                    match attr.parse_nested_meta(|meta| match meta.value() {
                        Ok(buf) => match buf.parse::<LitInt>() {
                            Ok(code) => {
                                error_code_attr = Some(code.base10_parse::<u16>().unwrap());
                                Ok(())
                            }
                            Err(err) => Ok(()),
                        },
                        Err(_) => Ok(()),
                    }) {
                        Ok(_) => (),
                        Err(err) => panic!("parse_nested_meta: {err} {:?}", attr),
                    };
                }

                let error_code = match error_code_attr {
                    Some(code) => quote! { #code },
                    None => quote! { 0 }, // Default to 0 if no error_code attribute is found
                };

                match &variant.fields {
                    Fields::Unit => {
                        quote_spanned! { variant.span() =>
                            Self::#variant_name => #error_code,
                        }
                    }
                    Fields::Named(_) | Fields::Unnamed(_) => {
                        // Handle tuple and struct variants
                        quote_spanned! { variant.span() =>
                            Self::#variant_name(_) => #error_code,
                        }
                    }
                }
            });

            let expanded = quote! {
                impl #impl_generics ErrorCode for #name #ty_generics #where_clause {
                    fn error_code(&self) -> i32 {
                        match self {
                            #(#variants)*
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        _ => panic!("ErrorCode can only be derived for enums"),
    }
}
