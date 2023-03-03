use crate::statistic::{
    statistic_consumer::{Statistic, StatisticConsumer},
    statistic_printer::StatisticPrinter,
};
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CallCounter {
    pub func_count: HashMap<String, u64>,
}

impl CallCounter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            func_count: HashMap::new(),
        }
    }
}

impl StatisticPrinter for CallCounter {
    fn print_statistic(&self) -> String {
        let mut table = Table::new();
        table.add_row(row!["Function", "Count"]);
        for (func_name, count) in self.func_count.iter() {
            table.add_row(row![func_name.green().bold(), count.blue()]);
        }
        format!("{}\n{}", "Number of calls".bright_yellow().bold(), table)
    }
}

impl StatisticConsumer for CallCounter {
    fn consume_statistic(&mut self, stat: &Statistic) {
        let count = self.func_count.entry(stat.func_name.clone()).or_insert(0);
        *count += 1;
    }

    fn clean_statistic(&mut self) {
        self.func_count.clear();
    }
}
