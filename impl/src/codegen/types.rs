use core::panic;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

type IsOptional = bool;

pub(crate) fn type_to_surreal_field_type(syn_type: &Type) -> Option<(TokenStream, IsOptional)> {
    match &syn_type {
        Type::Path(type_path) => match type_path.path.segments.len() {
            0 => panic!("Empty type path specified"),
            1 => {
                let path_segment = type_path.path.segments.first().unwrap();
                match path_segment.ident.to_string().as_str() {
                    "bool" => Some((quote! {surrealdb_schema_derive::surrealdb::sql::Kind::Bool}, false)),
                    "String" | "str" => {
                        Some((quote! {surrealdb_schema_derive::surrealdb::sql::Kind::String}, false))
                    }
                    "u8" | "u16" | "u32" | "u64" | "usize" => {
                        Some((quote! {surrealdb_schema_derive::surrealdb::sql::Kind::Int}, false))
                    }
                    "i8" | "i16" | "i32" | "i64" | "isize" => {
                        Some((quote! {surrealdb_schema_derive::surrealdb::sql::Kind::Int}, false))
                    }
                    "f32" | "f64" => {
                        Some((quote! {surrealdb_schema_derive::surrealdb::sql::Kind::Float}, false))
                    }
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(ref angle_args) =
                            path_segment.arguments
                        {
                            if let Some(syn::GenericArgument::Type(type_arg)) =
                                angle_args.args.first()
                            {
                                if let Some((inner_type, _)) = type_to_surreal_field_type(type_arg) {
                                    Some((inner_type, true))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            panic!("Found an Option without any type arguments")
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}
