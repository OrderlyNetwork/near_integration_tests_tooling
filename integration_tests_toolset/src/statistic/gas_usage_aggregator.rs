use super::statistic_consumer::{Statistic, StatisticConsumer};
use crate::tx_result::TxResultDetails;
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use std::collections::{BinaryHeap, HashMap};
use workspaces::types::Gas;

#[derive(Debug)]
pub struct OperationGasUsage {
    pub heap: BinaryHeap<Gas>,
}

#[derive(Debug)]
pub struct OperationGasStatistic {
    pub min: Gas,
    pub max: Gas,
    pub median: Gas,
}

impl From<&OperationGasUsage> for OperationGasStatistic {
    fn from(op_gas: &OperationGasUsage) -> Self {
        let gas_vec: Vec<Gas> = op_gas.heap.clone().into_sorted_vec();
        if gas_vec.is_empty() {
            return Self {
                min: 0,
                max: 0,
                median: 0,
            };
        } else {
            return Self {
                min: gas_vec.first().cloned().unwrap_or_default(),
                max: gas_vec.last().cloned().unwrap_or_default(),
                median: {
                    let mid = gas_vec.len() / 2;
                    if gas_vec.len() % 2 == 0 {
                        (gas_vec[mid] + gas_vec[mid - 1]) / 2
                    } else {
                        gas_vec[mid]
                    }
                },
            };
        }
    }
}

#[derive(Debug)]
pub struct GasUsage {
    pub func_gas: HashMap<String, OperationGasUsage>,
}

impl GasUsage {
    pub fn new() -> Self {
        Self {
            func_gas: HashMap::new(),
        }
    }
}

impl Default for GasUsage {
    fn default() -> Self {
        Self::new()
    }
}

trait GasPrinter {
    fn print_gas(&self) -> String;
}

impl GasPrinter for Gas {
    fn print_gas(&self) -> String {
        format!(
            "{:.3} {} ({:.6} {})",
            (*self as f64 / 1_000_000_000_000.).bright_magenta().bold(),
            "Tgas",
            (*self as f64 / 10_000_000_000_000_000.)
                .bright_magenta()
                .bold(),
            "NEAR"
        )
    }
}

impl StatisticConsumer for GasUsage {
    fn consume_statistic(&mut self, stat: Statistic) {
        if let TxResultDetails::Call(call_data) = stat.details {
            let op_gas = self
                .func_gas
                .entry(stat.func_name)
                .or_insert_with(|| OperationGasUsage {
                    heap: BinaryHeap::new(),
                });
            op_gas.heap.push(call_data.gas);
        }
    }

    fn print_statistic(&self) -> String {
        let mut table = Table::new();
        table.add_row(row!["Function", "Min", "Median", "Max"]);
        for (func, gas) in self.func_gas.iter() {
            let gas_stat = OperationGasStatistic::from(gas);
            table.add_row(row![
                func.green().bold(),
                gas_stat.min.print_gas(),
                gas_stat.median.print_gas(),
                gas_stat.max.print_gas()
            ]);
        }
        format!("{}", table)
    }

    fn clean_statistic(&mut self) {
        self.func_gas.clear();
    }
}
