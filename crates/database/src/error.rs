use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    #[error("Container already exists: {0}")]
    ContainerAlreadyExists(String),

    #[error("Invalid container data: {0}")]
    InvalidData(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Internal server error")]
    InternalError,
}

impl From<DbError> for actix_web::error::Error {
    fn from(err: DbError) -> Self {
        tracing::error!("Database error: {}", err);
        actix_web::error::ErrorInternalServerError("Database error")
    }
}
