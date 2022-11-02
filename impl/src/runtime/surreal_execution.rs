use anyhow::{bail, Result};
use surrealdb::{
    sql::{Query, Value},
    Datastore, Session,
};

use crate::SurrealValue;

pub async fn run_single_row_query<T: TryFrom<SurrealValue>>(
    datastore: &Datastore,
    session: &Session,
    query: Query,
) -> Result<Option<T>>
where
    <T as TryFrom<SurrealValue>>::Error: std::error::Error,
    <T as TryFrom<SurrealValue>>::Error: Send,
    <T as TryFrom<SurrealValue>>::Error: Sync,
    <T as TryFrom<SurrealValue>>::Error: 'static,
{
    let mut responses = datastore.process(query, session, None, true).await?;

    if responses.len() != 1 {
        bail!("Surrealdb returned the wrong number of query results");
    }
    let response = responses.remove(0);

    if let Ok(result_value) = response.result {
        if let Value::Array(mut result_array) = result_value {
            if result_array.len() == 0 {
                return Ok(None);
            }
            let first_value = result_array.remove(0);
            Ok(Some(SurrealValue(first_value).try_into()?))
        } else {
            bail!("SurrealDb returned a non-array value for a list query");
        }
    } else if let Err(surrealdb::Error::QueryEmpty) = response.result {
        Ok(None)
    } else if let Err(err) = response.result {
        Err(err.into())
    } else {
        panic!("Inaccessible code");
    }
}
