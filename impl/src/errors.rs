use thiserror::Error;

#[derive(Debug, Error)]
pub enum SurrealDbSchemaDeriveQueryError {
    #[error("Row object had no 'id' column")]
    RowObjectMissingIdColumn,

    #[error("SurrealDb Error: {0}")]
    SurrealDbError(surrealdb::Error),

    #[error("Value conversion failed")]
    InvalidValueTypeError(InvalidValueTypeError)
}

#[derive(Debug, Error)]
#[error("Expected {} but received {}", expected_type, received_type)]
pub struct InvalidValueTypeError {
    pub expected_type: String,
    pub received_type: String,
}

impl From<surrealdb::Error> for SurrealDbSchemaDeriveQueryError {
    fn from(error: surrealdb::Error) -> Self {
        SurrealDbSchemaDeriveQueryError::SurrealDbError(error)
    }
}

impl From<InvalidValueTypeError> for SurrealDbSchemaDeriveQueryError {
    fn from(error: InvalidValueTypeError) -> Self {
        SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(error)
    }
}