use async_trait::async_trait;
use workspaces::{result::ExecutionFinalResult, Account};

/// This trait defines the interface for the call method of the NEAR smart-contract
#[async_trait]
pub trait Call {
    /// Should be used to execute the prepared mutable call on the generated contract template method
    async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult>;
}
