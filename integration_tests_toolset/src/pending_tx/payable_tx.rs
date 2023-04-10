use super::call::Call;
use async_trait::async_trait;
use workspaces::{result::ExecutionFinalResult, Account, Contract};

/// Struct which encapsulates all required arguments to make a state-mutable call with the deposit to the NEAR smart-contract
#[derive(Debug)]
pub struct PayablePendingTx<'a> {
    contract: &'a Contract,
    function_name: String,
    // json structured args serialized to bytes
    args: Vec<u8>,
    // deposit provided by the user
    attached_deposit: u128,
}

impl<'a> PayablePendingTx<'a> {
    pub fn new(
        contract: &'a Contract,
        function_name: String,
        args: Vec<u8>,
        attached_deposit: u128,
    ) -> Self {
        Self {
            contract,
            function_name,
            args,
            attached_deposit,
        }
    }
}

#[async_trait]
impl<'a> Call for PayablePendingTx<'a> {
    async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), &self.function_name)
            .args(self.args)
            .deposit(self.attached_deposit)
            .max_gas()
            .transact()
            .await
    }
}
