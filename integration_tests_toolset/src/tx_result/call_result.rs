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

impl CallResult {
    pub fn value_from_res_for_promise<T: serde::de::DeserializeOwned>(
        res: &ExecutionFinalResult,
    ) -> Result<Option<T>> {
        let converted_type: Result<Option<T>> = res
            .clone()
            .into_result()?
            .json()
            .map(|el| Some(el))
            .map_err(|e| e.into());

        if res.is_success() && converted_type.is_err() {
            // In case PromiseOrValue contains successfully executed Promise
            // the deserialization to expected value type would fail. That is why
            // here None is populated
            Ok(None)
        } else {
            converted_type
        }
    }
}

impl<T> FromRes<T, ExecutionFinalResult> for CallResult
where
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
                receipt_failures: res.receipt_failures().into_iter().cloned().collect(),
                receipt_outcomes: res.receipt_outcomes().to_vec(),
            }),
        })
    }

    fn value_from_res(res: &ExecutionFinalResult) -> Result<T> {
        res.clone().into_result()?.json().map_err(|e| e.into())
    }
}
