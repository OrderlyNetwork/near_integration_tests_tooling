use super::statistic_consumer::{Statistic, StatisticConsumer};

/// Interface to consume the statistic group(like: tuple, vector, array, custom struct with multiple statistic fields, etc)
/// * Note: it should be implemented only for the statistic groups for single statistic use
pub trait StatisticGroupExt {
    // Populate statistic for provided consumers and create a general report into the result String
    fn process_statistic(self, consumers: &mut [&mut Box<dyn StatisticConsumer>]) -> String;

    fn populate_statistic<'a, 'b>(
        &self,
        consumers: &'a mut [&'b mut Box<dyn StatisticConsumer>],
    ) -> &'a mut [&'b mut Box<dyn StatisticConsumer>];
}

impl StatisticGroupExt for Vec<Statistic> {
    fn process_statistic(self, consumers: &mut [&mut Box<dyn StatisticConsumer>]) -> String {
        let mut result = String::new();

        consumers.iter_mut().for_each(|consumer| {
            self.iter().for_each(|statistic| {
                consumer.consume_statistic(&statistic);
            });
            result.push_str(&consumer.make_report());
        });

        result
    }

    fn populate_statistic<'a, 'b>(
        &self,
        consumers: &'a mut [&'b mut Box<dyn StatisticConsumer>],
    ) -> &'a mut [&'b mut Box<dyn StatisticConsumer>] {
        consumers.iter_mut().for_each(|consumer| {
            self.iter()
                .for_each(|stat| consumer.consume_statistic(stat));
        });

        consumers
    }
}

impl StatisticGroupExt for Statistic {
    fn process_statistic(self, consumers: &mut [&mut Box<dyn StatisticConsumer>]) -> String {
        let mut result = String::new();

        for consumer in consumers.iter_mut() {
            consumer.consume_statistic(&self);
            result.push_str(&consumer.make_report());
        }

        result
    }

    fn populate_statistic<'a, 'b>(
        &self,
        consumers: &'a mut [&'b mut Box<dyn StatisticConsumer>],
    ) -> &'a mut [&'b mut Box<dyn StatisticConsumer>] {
        consumers.iter_mut().for_each(|consumer| {
            consumer.consume_statistic(&self);
        });

        consumers
    }
}
