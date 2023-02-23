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

// Every entity which will work with statistics should implement this trait
pub trait StatisticConsumer: Sync + Send + std::fmt::Debug {
    fn consume_statistic(&mut self, stat: Statistic);
    fn print_statistic(&self) -> String;
    fn clean_statistic(&mut self);
}
