pub mod call_result;
pub mod log_parser;
pub mod view_result;

pub use self::{call_result::CallResult, view_result::ViewResult};
use crate::{
    error::Result,
    statistic::statistic_consumer::{Statistic, StatisticConsumer},
};

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
    pub fn populate_statistic<const N: usize>(
        self,
        consumers: &mut [Box<dyn StatisticConsumer>; N],
    ) -> Self {
        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&&Statistic::from(self.clone()));
        }
        self
    }

    #[allow(dead_code)]
    pub fn process_statistic<const N: usize>(
        self,
        mut consumers: [Box<dyn StatisticConsumer>; N],
    ) -> String {
        let mut result = String::new();

        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&Statistic::from(self.clone()));
            result.push_str(&consumer.print_statistic());
        }

        result
    }
}
