use proc_macro2::TokenStream;
use quote::{quote};
use syn::{FieldsNamed, Ident, Type, PathArguments, GenericArgument};

pub(crate) fn gen_try_from_surreal_value(
    struct_ident: &Ident,
    fields: &FieldsNamed,
) -> TokenStream {
    let field_definitions: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_ident = field.ident.clone().unwrap();
            if let Some(inner_type) = maybe_extract_optional(field) {
                quote! {
                    #field_ident: TryInto::<surrealdb_schema_derive::SurrealOption<#inner_type>>::try_into(
                        surrealdb_schema_derive::SurrealValue(
                            object_value.0.get(stringify!(name)).unwrap().clone(),
                        )
                    )?.into()
                }
            } else {
                quote! {
                    #field_ident: surrealdb_schema_derive::SurrealValue(
                        object_value.0.get(stringify!(#field_ident)).unwrap().clone()
                    ).try_into()?
                }
            }
        })
        .collect();

    quote! {
        impl TryFrom<surrealdb_schema_derive::SurrealValue> for #struct_ident {
            type Error = surrealdb_schema_derive::SurrealDbSchemaDeriveQueryError;

            fn try_from(value: surrealdb_schema_derive::SurrealValue) -> Result<Self, Self::Error> {
                if let surrealdb_schema_derive::surrealdb::sql::Value::Object(object_value) = value.0 {
                    Ok(Self {
                        #(#field_definitions),*
                    })
                } else {
                    Err(surrealdb_schema_derive::SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(
                        surrealdb_schema_derive::InvalidValueTypeError {
                            expected_type: stringify!(#struct_ident).into(),
                            received_type: value.0.to_string(),
                        }
                    ))
                }
            }
        }
    }
}

pub(crate) fn gen_into_surreal_value(struct_ident: &Ident, fields: &FieldsNamed) -> TokenStream {
    let field_conversions: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_ident = field.ident.clone().unwrap();
            let field_ref = if maybe_extract_optional(field).is_some() {
                quote! {surrealdb_schema_derive::SurrealOption(self.#field_ident)}
            } else {
                quote! {self.#field_ident}
            };
            quote! {
                (stringify!(#field_ident).into(), {
                    let surreal_value: surrealdb_schema_derive::SurrealValue = #field_ref.into();
                    surreal_value.into()
                })
            }
        })
        .collect();

    quote! {
        impl Into<surrealdb_schema_derive::SurrealValue> for #struct_ident {
            fn into(self) -> surrealdb_schema_derive::SurrealValue {
                surrealdb_schema_derive::SurrealValue(
                    surrealdb_schema_derive::surrealdb::sql::Value::Object(
                        surrealdb_schema_derive::surrealdb::sql::Object(std::collections::BTreeMap::from([
                            #(#field_conversions),*
                        ]))
                    )
                )
            }
        }
    }
}

fn maybe_extract_optional(field: &syn::Field) -> Option<Type> {
    if let Type::Path(ref path_type) = field.ty {
        if let Some(first) = path_type.path.segments.first() {
            if first.ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed) = &first.arguments {
                    let first_arg = angle_bracketed.args.first();
                    if let Some(GenericArgument::Type(inner_type)) = first_arg {
                        Some(inner_type.clone())
                    } else {
                        panic!("Invalid option: {:?}", path_type);
                    }
                } else {
                    panic!("Invalid option: {:?}", path_type);
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
