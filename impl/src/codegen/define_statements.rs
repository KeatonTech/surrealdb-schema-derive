use proc_macro2::{TokenStream, Ident};
use syn::{FieldsNamed, Field};
use quote::quote;
use crate::codegen::types;

pub(crate) fn gen_fn_get_field_definitions(fields: &FieldsNamed) -> TokenStream {
    let stmt_define_fields: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|f| gen_define_field_with_table_and_path_prefix(f))
        .collect();
    quote! {
        fn get_field_definitions(on_table: &String, path_prefix: &Vec<surrealdb_schema_derive::surrealdb::sql::Part>) -> surrealdb_schema_derive::surrealdb::sql::Statements {
            surrealdb_schema_derive::surrealdb::sql::Statements(vec![
                #(#stmt_define_fields.0),*
            ].into_iter().flatten().collect())
        }
    }
}

pub(crate) fn gen_fn_define_table(ident: &Ident) -> TokenStream {
    quote! {
        async fn define_table<'a>(args: &'a surrealdb_schema_derive::DefineTableArgs<'a>) -> surrealdb_schema_derive::anyhow::Result<()> {
            let mut statements = vec![];

            // Switch to the correct namespace, if requested
            if let Some(context) = &args.context {
                statements.push(surrealdb_schema_derive::surrealdb::sql::Statement::Use(
                    surrealdb_schema_derive::surrealdb::sql::statements::UseStatement {
                        ns: Some(context.namespace.to_string()),
                        db: Some(context.database.to_string())
                    }
                ));
            }

            // Define the table
            let table_name: String = stringify!(#ident).into();
            statements.push(surrealdb_schema_derive::surrealdb::sql::Statement::Define(
                surrealdb_schema_derive::surrealdb::sql::statements::DefineStatement::Table(
                    surrealdb_schema_derive::surrealdb::sql::statements::DefineTableStatement {
                        name: surrealdb::sql::Ident(table_name.clone()),
                        drop: false,
                        full: true,
                        view: None,
                        permissions: surrealdb_schema_derive::surrealdb::sql::Permissions::full()
                    }
                )
            ));

            // Define the fields of the table
            statements.extend(Self::get_field_definitions(&table_name, &vec![]).0);

            // Run the Query
            let responses = args.datastore.process(
                surrealdb_schema_derive::surrealdb::sql::Query(
                    surrealdb_schema_derive::surrealdb::sql::Statements(statements)
                ),
                args.session,
                None,
                true
            ).await?;

            // Test the result
            for response in responses {
                response.result?;
            }
            Ok(())
        }
    }
}

pub(crate) fn gen_define_field_with_table_and_path_prefix(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().expect("Field has no name");
    let new_path_prefix = quote! {
        {
            let mut path = path_prefix.clone();
            path.push(surrealdb_schema_derive::surrealdb::sql::Part::Field(stringify!(#ident).into()));
            path
        }
    };

    if let Some((surreal_type, is_optional)) = types::type_to_surreal_field_type(&field.ty) {
        let assert_property = if is_optional {
            quote!{Some(surrealdb::sql::Value::Expression(Box::new(
                surrealdb::sql::Expression { 
                    l: surrealdb::sql::Value::Idiom(
                        surrealdb_schema_derive::surrealdb::sql::Idiom(#new_path_prefix)
                    ), 
                    o: surrealdb::sql::Operator::NotEqual,
                    r: surrealdb::sql::Value::None
                }
            )))}
        } else {
            quote!{None}
        };
        quote! {
            surrealdb_schema_derive::surrealdb::sql::Statements(vec![
                surrealdb_schema_derive::surrealdb::sql::Statement::Define(
                    surrealdb_schema_derive::surrealdb::sql::statements::DefineStatement::Field(
                        surrealdb_schema_derive::surrealdb::sql::statements::DefineFieldStatement{
                            name: surrealdb_schema_derive::surrealdb::sql::Idiom(#new_path_prefix),
                            what: surrealdb_schema_derive::surrealdb::sql::Ident(on_table.to_string()),
                            kind: Some(#surreal_type),
                            value: None,
                            assert: #assert_property,
                            permissions: surrealdb_schema_derive::surrealdb::sql::Permissions::full()
                        }
                    )
                )
            ])
        }
    } else if let syn::Type::Path(child_type) = &field.ty {
        quote! {
            #child_type::get_field_definitions(on_table, &#new_path_prefix)
        }
    } else {
        panic!("Type {:?} is not supported", field.ty)
    }
}
