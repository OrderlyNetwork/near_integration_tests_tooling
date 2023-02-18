use anyhow::anyhow;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use test_token::TokenContractTest;
use workspaces::{types::Balance, Account, AccountId, Contract};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenInfo {
    pub account_id: AccountId,
    pub name: String,
    pub ticker: String,
    pub decimals: u8,
    pub storage_deposit_amount: Balance,
    pub wasm_file: Vec<u8>,
}

impl TokenInfo {
    pub fn parse(&self, amount: &str) -> anyhow::Result<Balance> {
        let mut amount = amount.replace(',', "").trim_start_matches('0').to_string();
        let width = if let Some(comma_pos) = amount.chars().position(|c| c == '.') {
            amount = amount.trim_end_matches('0').replace('.', "");
            let digits = amount.len();
            let width = comma_pos + self.decimals as usize;
            if width < digits {
                return Err(anyhow!("too many decimal places"));
            }
            width
        } else {
            amount.len() + self.decimals as usize
        };
        amount = format!("{:0<width$}", amount);
        Ok(amount.parse()?)
    }

    pub fn get_account_id(&self) -> near_sdk::AccountId {
        self.account_id.to_string().parse().unwrap()
    }
}

impl Display for TokenInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.account_id)
    }
}

impl AsRef<str> for TokenInfo {
    fn as_ref(&self) -> &str {
        self.account_id.as_str()
    }
}

static DEFAULT_FT_WASM: Lazy<Vec<u8>> =
    Lazy::new(|| include_bytes!("../../res/test_token.wasm").to_vec());

static WNEAR: Lazy<TokenInfo> = Lazy::new(|| TokenInfo {
    account_id: "wnear.test.near".parse().unwrap(),
    name: "wrapped Near".to_owned(),
    ticker: "wNEAR".to_owned(),
    decimals: 24,
    storage_deposit_amount: 100_000_000_000_000_000_000_000,
    wasm_file: DEFAULT_FT_WASM.clone(),
});

pub fn wnear() -> TokenInfo {
    WNEAR.clone()
}

static USDC: Lazy<TokenInfo> = Lazy::new(|| TokenInfo {
    account_id: "usdc.test.near".parse().unwrap(),
    name: "USD Coin".to_owned(),
    ticker: "USDC".to_owned(),
    decimals: 6,
    storage_deposit_amount: 100_000_000_000_000_000_000_000,
    wasm_file: DEFAULT_FT_WASM.clone(),
});

pub fn usdc() -> TokenInfo {
    USDC.clone()
}

static ETHER: Lazy<TokenInfo> = Lazy::new(|| TokenInfo {
    account_id: "eth.test.near".parse().unwrap(),
    name: "Ether".to_owned(),
    ticker: "ETH".to_owned(),
    decimals: 18,
    storage_deposit_amount: 100_000_000_000_000_000_000_000,
    wasm_file: DEFAULT_FT_WASM.clone(),
});

pub fn eth() -> TokenInfo {
    ETHER.clone()
}

pub struct TokenContracts {
    pub tokens: HashMap<TokenInfo, TokenContractTest>,
}

impl TokenContracts {
    pub fn token(&self, token: &TokenInfo) -> Option<&TokenContractTest> {
        self.tokens.get(token)
    }

    pub fn contract(&self, token: &TokenInfo) -> Option<&Contract> {
        self.tokens.get(token).map(|c| &c.contract)
    }

    pub fn account(&self, token: &TokenInfo) -> Option<&Account> {
        self.tokens.get(token).map(|c| c.contract.as_account())
    }
}
