use super::{
    mode_printer::ModePrinter,
    statistic_consumer::{Statistic, StatisticConsumer},
    statistic_printer::StatisticPrinter,
};
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use std::collections::{BinaryHeap, HashMap};

/// Struct for representing the range of values which shows the storage usage of the particular operation.
/// It should be used in scenarios with multiple storage measurements for the same operation.
#[derive(Debug)]
pub struct OperationStorageUsage {
    pub heap: BinaryHeap<i64>,
}

/// Struct for representing storage statistical values
#[derive(Debug)]
pub struct OperationStorageStatistic {
    pub min: i64,
    pub max: i64,
    pub median: i64,
}

impl From<&OperationStorageUsage> for OperationStorageStatistic {
    fn from(op_storage: &OperationStorageUsage) -> Self {
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

/// Struct for representing storage usage per each function
#[derive(Debug)]
pub struct StorageUsage {
    pub func_storage: HashMap<String, OperationStorageUsage>,
    mode_printer: ModePrinter,
}

impl StorageUsage {
    pub fn new(mode_printer: ModePrinter) -> Self {
        Self {
            func_storage: HashMap::new(),
            mode_printer,
        }
    }
}

impl Default for StorageUsage {
    fn default() -> Self {
        Self {
            func_storage: HashMap::new(),
            mode_printer: Default::default(),
        }
    }
}

/// Interface for storage usage printing
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

impl StatisticConsumer for StorageUsage {
    fn consume_statistic(&mut self, stat: &Statistic) {
        if let Some(storage_usage) = &stat.storage_usage {
            let op_storage = self
                .func_storage
                .entry(stat.func_name.clone())
                .or_insert_with(|| OperationStorageUsage {
                    heap: BinaryHeap::new(),
                });
            op_storage.heap.push(*storage_usage);
        }
    }

    fn clean_statistic(&mut self) {
        self.func_storage.clear();
    }
}

impl StatisticPrinter for StorageUsage {
    fn get_printer_mode(&self) -> &ModePrinter {
        &self.mode_printer
    }

    fn make_report(&self) -> String {
        let mut storage_stat_vec: Vec<_> = self
            .func_storage
            .iter()
            .map(|(func, storage)| {
                let storage_stat = OperationStorageStatistic::from(storage);
                (
                    func,
                    storage.heap.len(),
                    storage_stat.min,
                    storage_stat.median,
                    storage_stat.max,
                )
            })
            .collect();

        storage_stat_vec.sort_by(|a, b| b.3.cmp(&a.3));

        let mut table = Table::new();
        table.add_row(row!["Function", "Count", "Min", "Median", "Max"]);
        for (func, count, min, median, max) in storage_stat_vec.iter() {
            table.add_row(row![
                func.green().bold(),
                count.to_string().blue().bold(),
                min.print_storage(),
                median.print_storage(),
                max.print_storage()
            ]);
        }
        format!("{}\n{}", "Storage usage".bright_yellow().bold(), table)
    }
}
