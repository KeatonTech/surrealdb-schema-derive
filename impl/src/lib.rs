//! This crate implements the macro for `surrealdb_schema_derive` and should not be used directly.

mod codegen;
mod errors;
pub mod runtime;

use std::ops::Deref;

pub use runtime::surreal_value_primitives::*;
use codegen::*;
use async_trait::async_trait;
use derive_builder::Builder;
pub use errors::*;
use proc_macro2::TokenStream;
use quote::{quote};
pub use runtime::reference::SurrealReference;
pub use runtime::surreal_value_primitives::{SurrealValue, SurrealOption};
use surrealdb::{self, sql::{self, Id}, Datastore, Session};
use syn::{parse2, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed, Ident};
use anyhow::Result;

#[derive(Clone)]
pub struct DefineTableContext {
    pub namespace: String,
    pub database: String,
}

#[derive(Builder)]
pub struct DefineTableArgs<'a> {
    pub datastore: &'a Datastore,
    pub session: &'a Session,
    #[builder(setter(into, strip_option), default)]
    pub context: Option<DefineTableContext>,
}

pub trait SurrealDbObject:
    TryFrom<SurrealValue, Error = SurrealDbSchemaDeriveQueryError> + Into<SurrealValue>
{
    fn get_field_definitions(
        on_table: &String,
        path_prefix: &Vec<sql::Part>,
    ) -> surrealdb::sql::Statements;
}

#[async_trait]
pub trait SurrealDbTable: SurrealDbObject {
    type Row: std::ops::Deref<Target = Self>;

    async fn define_table<'a>(args: &'a DefineTableArgs<'a>) -> Result<()>;

    async fn fetch_from_datastore(
        id: String,
        datastore: &Datastore,
        session: &Session,
    ) -> Result<Option<Self::Row>>;

    async fn save_to_datastore(
        self,
        datastore: &Datastore,
        session: &Session,
    ) -> Result<Self::Row>;
}

pub trait SurrealDbRow: Deref<Target = Self::Table> + Into<Self::Table> where Self::Table: Into<SurrealValue> {
    type Table: SurrealDbTable;

    fn new(id: Id, value: Self::Table) -> Self;
    fn get_table_name() -> String;
    fn get_reference(&self) -> &SurrealReference<Self::Table>;
}

#[doc(hidden)]
pub fn derive_surreal_db_object(_item: TokenStream) -> Result<TokenStream, syn::Error> {
    let (struct_ident, fields) = extract_derive_struct(_item)?;
    gen_surreal_db_object(&struct_ident, &fields)
}

fn gen_surreal_db_object(
    struct_ident: &Ident,
    fields: &FieldsNamed,
) -> Result<TokenStream, syn::Error> {
    let impl_try_from_surreal_value =
        object_conversion::gen_try_from_surreal_value(&struct_ident, &fields);
    let impl_into_surreal_value = object_conversion::gen_into_surreal_value(&struct_ident, &fields);
    let fn_get_field_definitions = define_statements::gen_fn_get_field_definitions(&fields);
    return Ok(TokenStream::from(quote! {
        #impl_try_from_surreal_value
        #impl_into_surreal_value

        impl SurrealDbObject for #struct_ident {
            #fn_get_field_definitions
        }
    }));
}

#[doc(hidden)]
pub fn derive_surreal_db_table(_item: TokenStream) -> Result<TokenStream, syn::Error> {
    let (struct_ident, fields) = extract_derive_struct(_item)?;
    let surreal_db_object = gen_surreal_db_object(&struct_ident, &fields)?;
    let row_struct_name = quote::format_ident!("{}Row", struct_ident);
    let row_struct = row_struct::gen_row_struct(&struct_ident, &row_struct_name);
    let fn_define_table = define_statements::gen_fn_define_table(&struct_ident);
    let fn_fetch_from_datastore =
        crud_statements::gen_fn_fetch_from_datastore(&struct_ident);
    let fn_save_to_datastore =
        crud_statements::gen_fn_save_to_datastore(&struct_ident);
    return Ok(TokenStream::from(quote! {
        #surreal_db_object

        #row_struct

        #[surrealdb_schema_derive::async_trait::async_trait]
        impl SurrealDbTable for #struct_ident {
            type Row = #row_struct_name;
            #fn_define_table
            #fn_fetch_from_datastore
            #fn_save_to_datastore
        }
    }));
}

fn extract_derive_struct(
    struct_stream: TokenStream,
) -> Result<(Ident, FieldsNamed), syn::Error> {
    let top_level_error_span = struct_stream.span();
    let input: DeriveInput = parse2(struct_stream)?;
    let ident = input.ident;
    let fields = (if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(named_fields) = data_struct.fields {
            Ok(named_fields)
        } else {
            Err(syn::Error::new(
                top_level_error_span,
                "Must use named fields",
            ))
        }
    } else {
        Err(syn::Error::new(
            top_level_error_span,
            "Only structs are supported for surrealdb deriving",
        ))
    })?;
    Ok((ident, fields))
}

#[cfg(test)]
mod tests {
    use prettyplease;
    use syn::File;

    use super::*;

    #[test]
    fn derives_simple_struct() {
        assert!(parse2::<File>(
            derive_surreal_db_table(quote! {
                struct MyStruct {
                    name: String,
                    count: u8,
                    value: isize,
                }
            })
            .unwrap()
        )
        .is_ok());
        println!(
            "{}",
            prettyplease::unparse(
                &parse2::<File>(
                    derive_surreal_db_table(quote! {
                        struct RustStruct {
                            name: Option<String>,
                            generics: RustGenerics,
                        }
                    })
                    .unwrap()
                )
                .unwrap()
            )
        );
    }
}
