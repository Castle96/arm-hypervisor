use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ModelError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid container name: {0}")]
    InvalidContainerName(String),

    #[error("Invalid resource limit: {0}")]
    InvalidResourceLimit(String),

    #[error("Internal error")]
    InternalError,
}
