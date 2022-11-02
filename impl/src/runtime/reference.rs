use surrealdb::sql::{Id, Value, Thing};
use surrealdb::{Datastore, Session};

use crate::{InvalidValueTypeError, SurrealDbSchemaDeriveQueryError, SurrealValue};
use std::marker::PhantomData;
use anyhow::Result;
use crate::runtime::surreal_query_generation;
use crate::runtime::surreal_execution;

#[derive(Debug)]
pub struct SurrealReference<T> {
    table_name: String,
    id: Id,
    phantom_data: PhantomData<T>,
}

impl<T> Clone for SurrealReference<T> {
    fn clone(&self) -> Self {
        Self {
            table_name: self.table_name.clone(),
            id: self.id.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<T> Into<SurrealValue> for SurrealReference<T> {
    fn into(self) -> SurrealValue {
        SurrealValue(Value::Thing(Thing {
            tb: self.table_name,
            id: self.id,
        }))
    }
}

impl<T> TryFrom<SurrealValue> for SurrealReference<T> {
    type Error = SurrealDbSchemaDeriveQueryError;

    fn try_from(value: SurrealValue) -> Result<Self, Self::Error> {
        if let Value::Thing(thing) = value.0 {
            Ok(SurrealReference {
                table_name: thing.tb,
                id: thing.id,
                phantom_data: PhantomData,
            })
        } else {
            Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(
                InvalidValueTypeError {
                    expected_type: "thing".into(),
                    received_type: format!("{:?}", value.0),
                },
            ))
        }
    }
}

impl<T> SurrealReference<T>
where
    T: TryFrom<SurrealValue>,
    <T as TryFrom<SurrealValue>>::Error: std::error::Error,
    <T as TryFrom<SurrealValue>>::Error: Send,
    <T as TryFrom<SurrealValue>>::Error: Sync,
    <T as TryFrom<SurrealValue>>::Error: 'static,
{
    pub fn new(table_name: String, id: Id) -> SurrealReference<T> {
        SurrealReference {
            table_name: table_name,
            id: id,
            phantom_data: PhantomData,
        }
    }

    pub fn get_id(&self) -> Id {
        self.id.clone()
    }

    pub async fn resolve(
        &self,
        datastore: &Datastore,
        session: &Session,
    ) -> Result<Option<T>> {
        let query = surreal_query_generation::make_lookup_query(
            self.table_name.clone(),
            self.id.clone()
        );

        surreal_execution::run_single_row_query(datastore, session, query).await
    }
}
