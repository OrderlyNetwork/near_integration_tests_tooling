use super::statistic_consumer::{Statistic, StatisticConsumer};
use crate::tx_result::TxResultDetails;
use prettytable::{row, Table};
use std::collections::HashMap;
use workspaces::types::Gas;

// TODO: add min, max, median gas usage
#[derive(Debug)]
pub struct GasUsage {
    pub func_gas: HashMap<String, Gas>,
}

impl GasUsage {
    pub fn new() -> Self {
        Self {
            func_gas: HashMap::new(),
        }
    }
}

// TODO: implement async guard for HashMap
impl StatisticConsumer for GasUsage {
    fn consume_statistic(&mut self, stat: Statistic) {
        match stat.details {
            TxResultDetails::Call(call_data) => {
                self.func_gas.insert(stat.func_name, call_data.gas);
            }
            _ => {}
        }
    }

    fn print_statistic(&self) -> String {
        let mut table = Table::new();
        table.add_row(row!["Function", "Gas"]);
        for (func, gas) in self.func_gas.iter() {
            table.add_row(row![func, gas]);
        }
        format!("{}", table)
    }

    // TODO: add functions for returning statistics as structure and as table
}
