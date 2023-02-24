use crate::tx_result::{TxResult, TxResultDetails};

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

// TODO: add function print_customized_statistic with parameter, that format statistic output
// TODO: add function write_statistic(filename) - write statistic to file
// TODO: add function for returning statistics as structure
/// Every entity which will work with statistics should implement this trait
pub trait StatisticConsumer: Sync + Send + std::fmt::Debug {
    fn consume_statistic(&mut self, stat: Statistic);
    fn print_statistic(&self) -> String;
    fn clean_statistic(&mut self);
}
