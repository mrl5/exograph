use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("{0}")]
    Generic(String),

    #[error(transparent)]
    Delegate(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Ssl(#[from] openssl::error::ErrorStack),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("{0} {1}")]
    WithContext(String, #[source] Box<DatabaseError>),

    #[error(transparent)]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl DatabaseError {
    pub fn with_context(self, context: String) -> DatabaseError {
        DatabaseError::WithContext(context, Box::new(self))
    }
}

pub trait WithContext {
    fn with_context(self, context: String) -> Self;
}

impl<T> WithContext for Result<T, DatabaseError> {
    fn with_context(self, context: String) -> Result<T, DatabaseError> {
        self.map_err(|e| e.with_context(context))
    }
}