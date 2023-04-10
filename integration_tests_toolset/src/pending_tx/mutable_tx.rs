use super::call::Call;
use async_trait::async_trait;
use workspaces::{result::ExecutionFinalResult, Account, Contract};

/// Struct which encapsulates all required arguments to make a state-mutable call to the NEAR smart-contract
#[derive(Debug)]
pub struct MutablePendingTx<'a> {
    contract: &'a Contract,
    function_name: String,
    // json structured args serialized to bytes
    args: Vec<u8>,
}

impl<'a> MutablePendingTx<'a> {
    pub fn new(contract: &'a Contract, function_name: String, args: Vec<u8>) -> Self {
        Self {
            contract,
            function_name,
            args,
        }
    }
}

#[async_trait]
impl<'a> Call for MutablePendingTx<'a> {
    async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), &self.function_name)
            .args(self.args)
            .max_gas()
            .transact()
            .await
    }
}
