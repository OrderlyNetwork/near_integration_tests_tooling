use super::statistic_consumer::StatisticConsumer;
use crate::error::TestError;

/// Interface for printing aggregated statistic from multiple consumers(stored in some group like: array, vector, etc.)
pub trait StatisticGroupPrinter {
    fn print_statistic(&self) -> Result<(), TestError>;
}

impl<const N: usize> StatisticGroupPrinter for [Box<dyn StatisticConsumer>; N] {
    fn print_statistic(&self) -> Result<(), TestError> {
        for consumer in self {
            consumer.print_statistic()?;
        }

        Ok(())
    }
}
