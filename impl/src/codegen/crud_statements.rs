use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn gen_fn_fetch_from_datastore(table_name: &Ident) -> TokenStream {
    quote! {
        async fn fetch_from_datastore(id: String, datastore: &surrealdb_schema_derive::surrealdb::Datastore, session: &surrealdb_schema_derive::surrealdb::Session) -> surrealdb_schema_derive::anyhow::Result<Option<Self::Row>> {
            let query = surrealdb_schema_derive::runtime::make_lookup_query(
                stringify!(#table_name).into(),
                surrealdb_schema_derive::surrealdb::sql::Id::String(id.to_string()),
            );

            Ok(surrealdb_schema_derive::runtime::run_single_row_query(datastore, session, query).await?)
        }
    }
}

pub(crate) fn gen_fn_save_to_datastore(table_name: &Ident) -> TokenStream {
    quote! {
        async fn save_to_datastore(self, datastore: &surrealdb_schema_derive::surrealdb::Datastore, session: &surrealdb_schema_derive::surrealdb::Session) -> surrealdb_schema_derive::anyhow::Result<Self::Row> {
            let query = surrealdb_schema_derive::surrealdb::sql::Query(surrealdb_schema_derive::surrealdb::sql::Statements(vec![
                surrealdb_schema_derive::surrealdb::sql::Statement::Insert(surrealdb_schema_derive::surrealdb::sql::statements::InsertStatement {
                    into: surrealdb_schema_derive::surrealdb::sql::Table(stringify!(#table_name).into()),
                    data: surrealdb_schema_derive::runtime::to_single_row_values_expression(self.into())?,
                    ignore: false,
                    update: None,
                    output: Some(surrealdb_schema_derive::surrealdb::sql::Output::After),
                    timeout: None,
                    parallel: false,
                }),
            ]));

            Ok(surrealdb_schema_derive::runtime::run_single_row_query(datastore, session, query).await?.unwrap())
        }
    }
}
