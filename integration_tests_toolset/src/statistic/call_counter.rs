use super::mode_printer::ModePrinter;
use crate::statistic::{
    statistic_consumer::{Statistic, StatisticConsumer},
    statistic_processor::StatisticProcessor,
};
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CallCounter {
    pub func_count: HashMap<String, u64>,
    mode_printer: ModePrinter,
}

impl CallCounter {
    #[allow(dead_code)]
    pub fn new(mode_printer: ModePrinter) -> Self {
        Self {
            func_count: HashMap::new(),
            mode_printer,
        }
    }
}

impl Default for CallCounter {
    fn default() -> Self {
        Self {
            func_count: HashMap::new(),
            mode_printer: Default::default(),
        }
    }
}

impl StatisticProcessor for CallCounter {
    fn get_printer_mode(&self) -> &ModePrinter {
        &self.mode_printer
    }

    fn make_report(&self) -> String {
        let mut count_stat_vec: Vec<_> = self.func_count.iter().collect();

        count_stat_vec.sort_by(|a, b| b.1.cmp(&a.1));

        let mut table = Table::new();
        table.add_row(row!["Function", "Count"]);
        for (func_name, count) in count_stat_vec.iter() {
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
