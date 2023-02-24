use super::statistic_consumer::{Statistic, StatisticConsumer};
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug)]
pub struct OpertionStorageUsage {
    pub heap: BinaryHeap<i64>,
}

#[derive(Debug)]
pub struct OperationStorageStatistic {
    pub min: i64,
    pub max: i64,
    pub median: i64,
}

impl From<&OpertionStorageUsage> for OperationStorageStatistic {
    fn from(op_storage: &OpertionStorageUsage) -> Self {
        let storage_vec: Vec<i64> = op_storage.heap.clone().into_sorted_vec();
        if storage_vec.is_empty() {
            return Self {
                min: 0,
                max: 0,
                median: 0,
            };
        } else {
            return Self {
                min: storage_vec.first().cloned().unwrap_or_default(),
                max: storage_vec.last().cloned().unwrap_or_default(),
                median: {
                    let mid = storage_vec.len() / 2;
                    if storage_vec.len() % 2 == 0 {
                        (storage_vec[mid] + storage_vec[mid - 1]) / 2
                    } else {
                        storage_vec[mid]
                    }
                },
            };
        }
    }
}

#[derive(Debug)]
pub struct StorageUsage {
    pub func_storage: HashMap<String, OpertionStorageUsage>,
}

impl StorageUsage {
    pub fn new() -> Self {
        Self {
            func_storage: HashMap::new(),
        }
    }
}

impl Default for StorageUsage {
    fn default() -> Self {
        Self::new()
    }
}

trait StoragePrinter {
    fn print_storage(&self) -> String;
}

impl StoragePrinter for i64 {
    fn print_storage(&self) -> String {
        format!(
            "{:.3} {} ({:.5} {})",
            (*self).bright_magenta().bold(),
            "bytes",
            (*self as f64
                / (near_sdk::ONE_NEAR.saturating_div(near_sdk::env::storage_byte_cost()) as f64))
                .bright_magenta()
                .bold(),
            "NEAR"
        )
    }
}

// TODO: add gas byte cost to statistic

impl StatisticConsumer for StorageUsage {
    fn consume_statistic(&mut self, stat: Statistic) {
        if let Some(storage_usage) = stat.storage_usage {
            let op_storage =
                self.func_storage
                    .entry(stat.func_name)
                    .or_insert_with(|| OpertionStorageUsage {
                        heap: BinaryHeap::new(),
                    });
            op_storage.heap.push(storage_usage);
        }
    }

    fn print_statistic(&self) -> String {
        let mut table = Table::new();
        table.add_row(row!["Function", "Min", "Median", "Max"]);
        for (func, storage) in self.func_storage.iter() {
            let storage_stat = OperationStorageStatistic::from(storage);
            table.add_row(row![
                func.green().bold(),
                storage_stat.min.print_storage(),
                storage_stat.median.print_storage(),
                storage_stat.max.print_storage()
            ]);
        }
        format!("{}", table)
    }

    fn clean_statistic(&mut self) {
        self.func_storage.clear();
    }
}
