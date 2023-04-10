use crate::pending_tx::view::View;
use async_trait::async_trait;
use workspaces::{result::ViewResultDetails, Contract};

/// Struct which encapsulates all required arguments to make a view call of the NEAR smart-contract
#[derive(Debug)]
pub struct ImmutablePendingTx<'a> {
    contract: &'a Contract,
    function_name: String,
    // json structured args serialized to bytes
    args: Vec<u8>,
}

impl<'a> ImmutablePendingTx<'a> {
    pub fn new(contract: &'a Contract, function_name: String, args: Vec<u8>) -> Self {
        Self {
            contract,
            function_name,
            args,
        }
    }
}

#[async_trait]
impl<'a> View for ImmutablePendingTx<'a> {
    async fn view(self) -> workspaces::result::Result<ViewResultDetails> {
        self.contract
            .call(&self.function_name)
            .args(self.args)
            .view()
            .await
    }
}
