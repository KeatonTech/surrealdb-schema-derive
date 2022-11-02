//! This crate implements the macro for `surrealdb_schema_derive` and should not be used directly.
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Document your macro here.
#[proc_macro_derive(SurrealDbTable)]
pub fn derive_surreal_db_table(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match surrealdb_schema_derive_impl::derive_surreal_db_table(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

#[proc_macro_derive(SurrealDbObject)]
pub fn derive_surreal_db_object(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match surrealdb_schema_derive_impl::derive_surreal_db_object(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
