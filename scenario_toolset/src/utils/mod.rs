pub mod token_info;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use workspaces::{types::Balance, AccountId};

/// Convert workspaces::AccountId to near_sdk::AccountId
pub trait ToNearSdkAcc: ToString {
    fn to_near_sdk(&self) -> near_sdk::AccountId {
        self.to_string().parse().unwrap()
    }
}

impl ToNearSdkAcc for workspaces::AccountId {}

/// Common test account, that make payments
static MAKER_ID: Lazy<AccountId> = Lazy::new(|| "maker.test.near".parse().unwrap());

pub fn maker_id() -> AccountId {
    MAKER_ID.clone()
}

/// Common test account, that receive payments
static TAKER_ID: Lazy<AccountId> = Lazy::new(|| "taker.test.near".parse().unwrap());

pub fn taker_id() -> AccountId {
    TAKER_ID.clone()
}

/// Generate numbered account id with prefix
pub fn account_id(prefix: &str, index: u32) -> AccountId {
    match prefix {
        "" => format!("{index}.test.near").parse().unwrap(),
        prefix => format!("{prefix}_{index}.test.near").parse().unwrap(),
    }
}

/// Define account id with tokens mint amounts
/// Using to initialize test accounts in initialize_context
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
