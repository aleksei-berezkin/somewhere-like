use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, PathArguments, Type};

/// Generates a struct where `Vec<String>` is replaced with `|`-concatenated String.
/// Input is validated to be non-empty and not contain `|`.
/// Vec of other types are not supported.
#[proc_macro_derive(CsvFriendly)]
pub fn csv_friendly(token_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(token_input as DeriveInput);

    let orig_name = &input.ident;
    let csv_friendly_name = format_ident!("{}CsvFriendly", orig_name);

    let orig_fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(named_fields) = &data_struct.fields {
            &named_fields.named
        } else {
            unimplemented!("Only named fields are supported")
        }
    } else {
        unimplemented!("Only structs are supported")
    };

    let csv_friendly_fields = orig_fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        let new_ty = match get_vec_type(ty) {
            VecType::NoVec => quote! { #ty },
            VecType::String => quote! { String },
            VecType::Other => unimplemented!("{} is not supported", quote! { #ty }),
        };

        quote! {
            pub #name: #new_ty
        }
    });

    let from_orig_fields = orig_fields.iter().map(|field| {
        let name = &field.ident;

        let assignment_rhs = match get_vec_type(&field.ty) {
            VecType::NoVec => quote! { orig.#name },
            VecType::String => quote! { validate_and_join(orig.#name) },
            VecType::Other => unreachable!(),
        };

        quote! {
            #name: #assignment_rhs
        }
    });

    let from_csv_friendly_fields = orig_fields.iter().map(|field| {
        let name = &field.ident;

        let assignment_rhs = match get_vec_type(&field.ty) {
            VecType::NoVec => quote! { csv_friendly.#name },
            VecType::String => quote! { csv_friendly.#name.split("|").map(|s| s.into()).collect() },
            VecType::Other => unreachable!(),
        };

        quote! {
            #name: #assignment_rhs
        }
    });

    let generated = quote! {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct #csv_friendly_name {
            #(#csv_friendly_fields,)*
        }

        impl From<#orig_name> for #csv_friendly_name {
            fn from(orig: #orig_name) -> Self {
                let validate_and_join = |v: Vec<String>| -> String {
                    if v.is_empty() {
                        panic!("Empty Vec is not supported because it serializes to the same string as [\"\"]")
                    }
                    v.iter().for_each(|s| {
                        if s.contains("|") { panic!("\"{}\" contains a delimiter \"|\"", s) }
                    });
                    v.join("|")
                };
                Self {
                    #(#from_orig_fields,)*
                }
            }
        }

        impl From<#csv_friendly_name> for #orig_name {
            fn from(csv_friendly: #csv_friendly_name) -> Self {
                Self {
                    #(#from_csv_friendly_fields,)*
                }
            }
        }
    };

    generated.into()
}

enum VecType {
    NoVec,
    String,
    Other
}

fn get_vec_type(ty: &Type) -> VecType {
    if let Type::Path(type_path) = ty {
        let parts = &type_path.path.segments;
        if parts.len() == 1 && parts[0].ident == "Vec" {
            if let PathArguments::AngleBracketed(args) = &parts[0].arguments {
                if let Some(syn::GenericArgument::Type(Type::Path(inner))) = args.args.first() {
                    if inner.path.is_ident("String") {
                        return VecType::String
                    }
                }
            }
            return VecType::Other;
        }
    }
    return VecType::NoVec
}
