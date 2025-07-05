use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type AppResult<T> = Result<T, AppError>;