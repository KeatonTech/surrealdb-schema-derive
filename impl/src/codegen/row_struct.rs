use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) fn gen_row_struct(
    struct_ident: &Ident,
    row_struct_name: &Ident,
) -> TokenStream {
    quote! {
        #[derive(Debug)]
        struct #row_struct_name {
            surreal_ref: surrealdb_schema_derive::SurrealReference<#struct_ident>,
            value: #struct_ident
        }

        impl std::ops::Deref for #row_struct_name {
            type Target = #struct_ident;

            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }

        impl Into<#struct_ident> for #row_struct_name {
            fn into(self) -> #struct_ident {
                self.value
            }
        }

        impl Into<surrealdb_schema_derive::SurrealValue> for #row_struct_name {
            fn into(self) -> surrealdb_schema_derive::SurrealValue {
                surrealdb_schema_derive::runtime::row_to_surreal_value(self)
            }
        }

        impl TryFrom<surrealdb_schema_derive::SurrealValue> for #row_struct_name {
            type Error = surrealdb_schema_derive::SurrealDbSchemaDeriveQueryError;

            fn try_from(value: surrealdb_schema_derive::SurrealValue) -> Result<Self, Self::Error> {
                surrealdb_schema_derive::runtime::surreal_value_to_row(value)
            }
        }

        impl surrealdb_schema_derive::SurrealDbRow for #row_struct_name {
            type Table = #struct_ident;

            fn new(id: surrealdb_schema_derive::surrealdb::sql::Id, value: Self::Table) -> Self {
                #row_struct_name {
                    surreal_ref: surrealdb_schema_derive::SurrealReference::new(stringify!(#struct_ident).into(), id),
                    value: value
                }
            }

            fn get_table_name() -> String {
                stringify!(#struct_ident).into()
            }

            fn get_reference(&self) -> &surrealdb_schema_derive::SurrealReference<Self::Table> {
                &self.surreal_ref
            }
        }
    }
}
