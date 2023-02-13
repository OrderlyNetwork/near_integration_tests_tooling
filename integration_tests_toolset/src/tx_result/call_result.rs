use super::{FromRes, Result, TxResult, TxResultDetails};
use workspaces::{
    result::{ExecutionFinalResult, ExecutionOutcome},
    types::Gas,
};

#[derive(Debug, Clone)]
pub struct CallResult {
    pub gas: Gas,
    pub receipt_failures: Vec<ExecutionOutcome>,
    pub receipt_outcomes: Vec<ExecutionOutcome>,
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
