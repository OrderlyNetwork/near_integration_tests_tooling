pub mod call_result;
pub mod view_result;

pub use self::{call_result::CallResult, view_result::ViewResult};
use crate::{error::Result, statistic::statistic_consumer::StatisticConsumer};

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

pub trait FromRes<T, R> {
    fn value_from_res(res: &R) -> Result<T>;
    fn from_res(
        func_name: String,
        value: T,
        storage_usage: Option<i64>,
        res: R,
    ) -> Result<TxResult<T>>;
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
