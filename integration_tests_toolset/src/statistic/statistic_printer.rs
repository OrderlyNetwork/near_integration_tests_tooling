use super::statistic_consumer::StatisticConsumer;

pub trait StatisticPrinter {
    fn print_statistic(&self) -> String;
}

impl<const N: usize> StatisticPrinter for [Box<dyn StatisticConsumer>; N] {
    fn print_statistic(&self) -> String {
        let mut result = String::new();

        for consumer in self.iter() {
            result.push_str(&consumer.print_statistic());
        }

        result
    }
}
