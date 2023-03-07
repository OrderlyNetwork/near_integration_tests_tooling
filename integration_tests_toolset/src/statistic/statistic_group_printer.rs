use crate::error::TestError;

use super::statistic_consumer::StatisticConsumer;

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
