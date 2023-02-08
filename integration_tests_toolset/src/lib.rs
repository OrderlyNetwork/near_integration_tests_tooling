#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod toolset {
    use async_trait::async_trait;
    use thiserror::Error;
    use workspaces::{
        result::{ExecutionFinalResult, ExecutionResult, Value, ViewResultDetails},
        Account, Contract,
    };

    #[derive(Debug)]
    pub struct MutablePendingTx<'a> {
        contract: &'a Contract,
        function_name: String,
        args: Vec<u8>,
    }

    #[derive(Debug)]
    pub struct ImmutablePendingTx<'a> {
        contract: &'a Contract,
        function_name: String,
        args: Vec<u8>,
    }

    #[derive(Debug)]
    pub struct PayablePendingTx<'a> {
        contract: &'a Contract,
        function_name: String,
        args: Vec<u8>,
        attached_deposit: u128,
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

    impl<'a> MutablePendingTx<'a> {
        pub fn new(contract: &'a Contract, function_name: String, args: Vec<u8>) -> Self {
            Self {
                contract,
                function_name,
                args,
            }
        }
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
    pub trait View {
        async fn view(self) -> workspaces::result::Result<ViewResultDetails>;
    }

    #[async_trait]
    pub trait Call {
        async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult>;
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

    #[async_trait]
    impl<'a> Call for MutablePendingTx<'a> {
        async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult> {
            caller
                .call(&self.contract.id(), &self.function_name)
                .args(self.args)
                .max_gas()
                .transact()
                .await
        }
    }

    #[async_trait]
    impl<'a> Call for PayablePendingTx<'a> {
        async fn call(self, caller: &Account) -> workspaces::result::Result<ExecutionFinalResult> {
            caller
                .call(&self.contract.id(), &self.function_name)
                .args(self.args)
                .deposit(self.attached_deposit)
                .max_gas()
                .transact()
                .await
        }
    }

    #[derive(Debug)]
    pub struct ViewResult<T> {
        pub value: T,
        pub res: ViewResultDetails,
    }

    #[derive(Debug)]
    pub struct CallResult<T> {
        pub value: T,
        pub res: ExecutionResult<Value>,
    }

    #[derive(Debug, Error)]
    pub enum TestError {
        #[error("Workspace error: {:?}", _0)]
        Workspace(#[from] workspaces::error::Error),
        #[error("Execution failure: {}", _0)]
        ExecutionFailure(#[from] workspaces::result::ExecutionFailure),
        #[error("Internal receipt failure: {:?}", _0)]
        ReceiptFailure(#[from] workspaces::error::ErrorKind),
    }

    pub type Result<T> = std::result::Result<T, TestError>;
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub use toolset::*;
