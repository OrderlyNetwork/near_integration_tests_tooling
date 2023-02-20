use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Workspace error: {:?}", _0)]
    Workspace(#[from] Box<workspaces::error::Error>),
    #[error("Execution failure: {}", _0)]
    ExecutionFailure(#[from] Box<workspaces::result::ExecutionFailure>),
    #[error("Internal receipt failure: {:?}", _0)]
    ReceiptFailure(#[from] Box<workspaces::error::ErrorKind>),
    #[error("Test error: {}", _0)]
    Custom(String),
}

impl From<workspaces::error::Error> for TestError {
    fn from(error: workspaces::error::Error) -> Self {
        TestError::Workspace(Box::new(error))
    }
}

impl From<workspaces::result::ExecutionFailure> for TestError {
    fn from(failure: workspaces::result::ExecutionFailure) -> Self {
        TestError::ExecutionFailure(Box::new(failure))
    }
}

impl From<workspaces::error::ErrorKind> for TestError {
    fn from(error: workspaces::error::ErrorKind) -> Self {
        TestError::ReceiptFailure(Box::new(error))
    }
}

pub type Result<T> = std::result::Result<T, TestError>;
