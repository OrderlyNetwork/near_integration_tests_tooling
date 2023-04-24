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
    pub fn populate_statistic(self, consumers: &mut [&mut Box<dyn StatisticConsumer>]) -> Self {
        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&Statistic::from(self.clone()));
        }
        self
    }

    #[allow(dead_code)]
    pub fn process_statistics<const N: usize>(
        self,
        consumers: &mut [&mut Box<dyn StatisticConsumer>],
    ) -> String {
        let mut result = String::new();

        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&Statistic::from(self.clone()));
            result.push_str(&consumer.make_report());
        }

        result
    }
}

pub trait IntoMutRefs<T, const N: usize> {
    fn into_refs(&mut self) -> [&mut T; N];
}

impl<const N: usize> IntoMutRefs<Box<dyn StatisticConsumer>, N>
    for [Box<dyn StatisticConsumer>; N]
{
    fn into_refs(&mut self) -> [&mut Box<dyn StatisticConsumer>; N] {
        self.iter_mut()
            .map(|c| c)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}
