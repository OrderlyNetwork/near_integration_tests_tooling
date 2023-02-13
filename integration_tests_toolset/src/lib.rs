#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod toolset {
    use async_trait::async_trait;
    use owo_colors::OwoColorize;
    use prettytable::{row, Table};
    use std::collections::HashMap;
    use thiserror::Error;
    use workspaces::{
        result::{ExecutionFinalResult, ExecutionOutcome, ViewResultDetails},
        types::Gas,
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

    #[derive(Debug, Clone)]
    pub struct TxResult<T> {
        pub func_name: String,
        pub value: T,
        pub storage_usage: Option<i64>,
        pub details: TxResultDetails,
    }

    #[derive(Debug, Clone)]
    pub enum TxResultDetails {
        View(ViewResult),
        Call(CallResult),
    }

    impl Default for TxResultDetails {
        fn default() -> Self {
            Self::View(ViewResult { logs: vec![] })
        }
    }

    #[derive(Debug, Clone)]
    pub struct ViewResult {
        pub logs: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct CallResult {
        pub gas: Gas,
        pub receipt_failures: Vec<ExecutionOutcome>,
        pub receipt_outcomes: Vec<ExecutionOutcome>,
    }

    pub trait FromRes<T, R> {
        fn value_from_res(res: &R) -> Result<T>;
        fn from_res(
            func_name: String,
            value: T,
            storage_usage: Option<i64>,
            res: R,
        ) -> Result<TxResult<T>>;
    }

    impl<T> FromRes<T, ViewResultDetails> for ViewResult
    where
        T: serde::de::DeserializeOwned,
    {
        fn from_res(
            func_name: String,
            value: T,
            storage_usage: Option<i64>,
            res: ViewResultDetails,
        ) -> Result<TxResult<T>> {
            Ok(TxResult {
                func_name,
                value,
                storage_usage,
                details: TxResultDetails::View(ViewResult { logs: res.logs }),
            })
        }

        fn value_from_res(res: &ViewResultDetails) -> Result<T> {
            res.json().map_err(|e| e.into())
        }
    }

    impl<T> FromRes<T, ExecutionFinalResult> for CallResult
    where
        // TODO: decide what to do with non-Deserializable types especially PromiseOrValue<U128>
        // that returns from ft_transfer_call or internal contract calls
        T: serde::de::DeserializeOwned,
    {
        fn from_res(
            func_name: String,
            value: T,
            storage_usage: Option<i64>,
            res: ExecutionFinalResult,
        ) -> Result<TxResult<T>> {
            Ok(TxResult {
                func_name,
                value,
                storage_usage,
                details: TxResultDetails::Call(CallResult {
                    gas: res.total_gas_burnt,
                    receipt_failures: res
                        .receipt_failures()
                        .into_iter()
                        .map(|f| f.clone())
                        .collect(),
                    receipt_outcomes: res
                        .receipt_outcomes()
                        .into_iter()
                        .map(|f| f.clone())
                        .collect(),
                }),
            })
        }

        fn value_from_res(res: &ExecutionFinalResult) -> Result<T> {
            res.clone().into_result()?.json().map_err(|e| e.into())
        }
    }

    pub trait ResLogger<R> {
        fn check_res_log_failures(&self) -> Result<()>;
    }

    impl ResLogger<ViewResultDetails> for ViewResultDetails {
        fn check_res_log_failures(&self) -> Result<()> {
            Ok(())
        }
    }

    macro_rules! print_log {
        ( $x:expr, $($y:expr),+ ) => {
            let thread_name = std::thread::current().name().unwrap().to_string();
            if thread_name == "main" {
                println!($x, $($y),+);
            } else {
                println!(
                    concat!("{}\n    ", $x),
                    thread_name.bold(),
                    $($y),+
                );
            }
        };
    }

    impl ResLogger<ExecutionFinalResult> for ExecutionFinalResult {
        fn check_res_log_failures(&self) -> Result<()> {
            for failure in self.receipt_failures() {
                // TODO: rize exception if internal receipt failures
                print_log!("{:#?}", failure.bright_red());
            }
            Ok({
                self.clone().into_result()?;
            })
        }
    }

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

    #[derive(Debug, Clone, Default)]
    pub struct Statistic {
        pub func_name: String,
        pub storage_usage: Option<i64>,
        pub details: TxResultDetails,
    }

    impl<T> From<TxResult<T>> for Statistic {
        fn from(tx_res: TxResult<T>) -> Self {
            Statistic {
                func_name: tx_res.func_name,
                storage_usage: tx_res.storage_usage,
                details: tx_res.details,
            }
        }
    }

    impl<T> TxResult<T>
    where
        T: Clone,
    {
        pub fn populate_statistic(self, consumers: &mut [&mut impl StatisticConsumer]) -> Self {
            consumers.into_iter().for_each(|con| {
                con.consume_statistic(self.clone().into());
            });
            self
        }
    }

    // Every entity which will work with statistics should implement this trait
    pub trait StatisticConsumer: Sync + Send + 'static + std::fmt::Debug {
        fn consume_statistic(&mut self, stat: Statistic);
        fn print_statistic(&self) -> String;
    }

    // TODO: add min, max, median gas usage
    #[derive(Debug)]
    pub struct GasUsage {
        pub func_gas: HashMap<String, Gas>,
    }

    impl GasUsage {
        pub fn new() -> Self {
            Self {
                func_gas: HashMap::new(),
            }
        }
    }

    // TODO: implement async guard for HashMap
    impl StatisticConsumer for GasUsage {
        fn consume_statistic(&mut self, stat: Statistic) {
            match stat.details {
                TxResultDetails::Call(call_data) => {
                    self.func_gas.insert(stat.func_name, call_data.gas);
                }
                _ => {}
            }
        }

        fn print_statistic(&self) -> String {
            let mut table = Table::new();
            table.add_row(row!["Function", "Gas"]);
            for (func, gas) in self.func_gas.iter() {
                table.add_row(row![func, gas]);
            }
            format!("{}", table)
        }

        // TODO: add functions for returning statistics as structure and as table
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub use toolset::*;
