use super::statistic_consumer::{Statistic, StatisticConsumer};

pub trait StatisticGroupExt {
    fn process_statistic<const N: usize>(
        self,
        consumers: [Box<dyn StatisticConsumer>; N],
    ) -> String;

    fn populate_statistic<const N: usize>(
        &self,
        consumers: [Box<dyn StatisticConsumer>; N],
    ) -> [Box<dyn StatisticConsumer>; N];
}

impl StatisticGroupExt for Vec<Statistic> {
    fn process_statistic<const N: usize>(
        self,
        mut consumers: [Box<dyn StatisticConsumer>; N],
    ) -> String {
        let mut result = String::new();

        for consumer in consumers.iter_mut() {
            for statistic in self.iter() {
                consumer.consume_statistic(&statistic);
            }
            result.push_str(&consumer.print_statistic());
        }

        result
    }

    fn populate_statistic<const N: usize>(
        &self,
        consumers: [Box<dyn StatisticConsumer>; N],
    ) -> [Box<dyn StatisticConsumer>; N] {
        consumers.map(|mut consumer| {
            self.iter()
                .for_each(|stat| consumer.consume_statistic(stat));
            consumer
        })
    }
}

impl StatisticGroupExt for Statistic {
    fn process_statistic<const N: usize>(
        self,
        mut consumers: [Box<dyn StatisticConsumer>; N],
    ) -> String {
        let mut result = String::new();

        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&self);
            result.push_str(&consumer.print_statistic());
        }

        result
    }

    fn populate_statistic<const N: usize>(
        &self,
        consumers: [Box<dyn StatisticConsumer>; N],
    ) -> [Box<dyn StatisticConsumer>; N] {
        consumers.map(|mut consumer| {
            consumer.consume_statistic(&self);
            consumer
        })
    }
}
