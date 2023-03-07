pub mod token_info;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use workspaces::{types::Balance, AccountId};

#[macro_export]
macro_rules! print_log {
    ( $x:expr, $($y:expr),+ ) => {
        let thread_name = std::thread::current().name().unwrap().to_string();
        if thread_name == "main" {
            println!($x, $($y),+);
        } else {
            println!(
                concat!("{}\n    ", $x),
                thread_name.bold(),
                $($y),+
            );
        }
    };
}

pub trait ToNearSdkAcc: ToString {
    fn to_near_sdk(&self) -> near_sdk::AccountId {
        self.to_string().parse().unwrap()
    }
}

impl ToNearSdkAcc for workspaces::AccountId {}

static MAKER_ID: Lazy<AccountId> = Lazy::new(|| "maker.test.near".parse().unwrap());

pub fn maker_id() -> AccountId {
    MAKER_ID.clone()
}

static TAKER_ID: Lazy<AccountId> = Lazy::new(|| "taker.test.near".parse().unwrap());

pub fn taker_id() -> AccountId {
    TAKER_ID.clone()
}

pub fn account_id(prefix: &str, index: u32) -> AccountId {
    match prefix {
        "" => format!("{index}.test.near").parse().unwrap(),
        prefix => format!("{prefix}_{index}.test.near").parse().unwrap(),
    }
}

#[derive(Debug, Clone)]
pub struct TestAccount {
    pub account_id: AccountId,
    pub mint_amount: HashMap<String, Balance>,
}

impl Default for TestAccount {
    fn default() -> Self {
        Self {
            account_id: "account.test.near".parse().unwrap(),
            mint_amount: HashMap::new(),
        }
    }
}
