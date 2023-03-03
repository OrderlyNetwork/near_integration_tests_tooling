use super::{FromRes, TxResult, TxResultDetails};
use crate::error::Result;
use workspaces::result::ViewResultDetails;

#[derive(Debug, Clone)]
pub struct ViewResult {
    pub logs: Vec<String>,
}

impl<T> FromRes<T, ViewResultDetails> for ViewResult
where
    T: serde::de::DeserializeOwned,
{
    // TODO in case view call will return PromiseOrValue it will be required to add handling here
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
