use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Workspace error: {:?}", _0)]
    Workspace(#[from] workspaces::error::Error),
    #[error("Execution failure: {}", _0)]
    ExecutionFailure(#[from] workspaces::result::ExecutionFailure),
    #[error("Internal receipt failure: {:?}", _0)]
    ReceiptFailure(#[from] workspaces::error::ErrorKind),
    #[error("Test error: {}", _0)]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, TestError>;
