use surrealdb::sql::{
    statements, Field, Fields, Id, Idiom, Part, Query, Statement, Statements, Thing, Value,
    Values,
};

use crate::SurrealValue;

pub fn make_lookup_query(table_name: String, id: Id) -> Query {
    Query(Statements(vec![Statement::Select(
        statements::SelectStatement {
            expr: Fields(vec![Field::All]),
            what: Values(vec![Value::Thing(Thing {
                tb: table_name,
                id: id,
            })]),
            cond: None,
            split: None,
            group: None,
            order: None,
            limit: None,
            start: None,
            fetch: None,
            version: None,
            timeout: None,
            parallel: false,
        },
    )]))
}

pub fn to_single_row_values_expression(
    surreal_object: SurrealValue,
) -> Result<surrealdb::sql::Data, surrealdb::Error> {
    let mut inner: Vec<(Idiom, Value)> = vec![];

    if let surrealdb::sql::Value::Object(object_value) = surreal_object.0 {
        for (key, value) in object_value {
            inner.push((Idiom(vec![Part::Field(surrealdb::sql::Ident(key))]), value))
        }
    } else {
        return Err(surrealdb::Error::Ignore);
    }

    let outer = vec![inner];
    return Ok(surrealdb::sql::Data::ValuesExpression(outer));
}
