#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use async_trait::async_trait;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::{future::Future, pin::Pin};
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use workspaces::{
    result::{ExecutionFinalResult, ViewResultDetails},
    Account, Contract,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct MutablePendingTx {
    function_name: String,
    args: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ImmutablePendingTx {
    function_name: String,
    args: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PayablePendingTx {
    function_name: String,
    args: Vec<u8>,
    attached_deposit: u128,
}

impl ImmutablePendingTx {
    pub fn new(function_name: String, args: Vec<u8>) -> Self {
        Self {
            function_name,
            args,
        }
    }
}

impl MutablePendingTx {
    pub fn new(function_name: String, args: Vec<u8>) -> Self {
        Self {
            function_name,
            args,
        }
    }
}

impl PayablePendingTx {
    pub fn new(function_name: String, args: Vec<u8>, attached_deposit: u128) -> Self {
        Self {
            function_name,
            args,
            attached_deposit,
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub trait View {
    fn view<'a>(
        self,
        contract: &'a Contract,
    ) -> Pin<Box<dyn Future<Output = workspaces::result::Result<ViewResultDetails>> + Send + 'a>>;
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[async_trait]
pub trait Call {
    async fn call<'a>(
        self,
        contract: &Contract,
        caller: &Account,
    ) -> workspaces::result::Result<ExecutionFinalResult>;
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
impl View for ImmutablePendingTx {
    fn view<'a>(
        self,
        contract: &'a Contract,
    ) -> Pin<Box<dyn Future<Output = workspaces::result::Result<ViewResultDetails>> + Send + 'a>>
    {
        async fn run(
            pending_tx: ImmutablePendingTx,
            contract: &Contract,
        ) -> workspaces::result::Result<ViewResultDetails> {
            contract
                .call(&pending_tx.function_name)
                .args(pending_tx.args)
                .view()
                .await
        }

        Box::pin(run(self, contract))
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[async_trait]
impl Call for MutablePendingTx {
    async fn call<'a>(
        self,
        contract: &Contract,
        caller: &Account,
    ) -> workspaces::result::Result<ExecutionFinalResult> {
        caller
            .call(contract.id(), &self.function_name)
            .args(self.args)
            .max_gas()
            .transact()
            .await
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[async_trait]
impl Call for PayablePendingTx {
    async fn call<'a>(
        self,
        contract: &Contract,
        caller: &Account,
    ) -> workspaces::result::Result<ExecutionFinalResult> {
        caller
            .call(contract.id(), &self.function_name)
            .args(self.args)
            .deposit(self.attached_deposit)
            .max_gas()
            .transact()
            .await
    }
}
