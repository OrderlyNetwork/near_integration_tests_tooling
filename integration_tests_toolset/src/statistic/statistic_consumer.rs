use super::statistic_printer::StatisticPrinter;
use crate::tx_result::{TxResult, TxResultDetails};

/// This struct aggregates all required statistic data related to the smart-contract method call
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

/// Trait which should be used for the statistic aggregation
/// Implementor consumes statistic related to particular smart-contract methods
/// Every entity which will need to aggregate statistics should implement this trait
/// * Note: also statistic could be cleaned at any stage of the test scenario
pub trait StatisticConsumer: Sync + Send + std::fmt::Debug + StatisticPrinter {
    // Interface method for populating consumer with the statistic
    fn consume_statistic(&mut self, stat: &Statistic);
    fn clean_statistic(&mut self);
}
