use async_trait::async_trait;
use workspaces::{result::ExecutionFinalResult, Account};

#[async_trait]
pub trait Call {
    async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult>;
}
