use std::fmt;

use integration_tests_bindgen_macro::integration_tests_bindgen;
use near_contract_standards::{
    fungible_token::{
        core::FungibleTokenCore,
        metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
        resolver::FungibleTokenResolver,
        FungibleToken,
    },
    storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement},
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
};

#[integration_tests_bindgen(internal)]
#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct TokenContract {
    name: String,
    symbol: String,
    decimals: u8,
    token: FungibleToken,
}

impl fmt::Debug for TokenContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenContract")
            .field("name", &self.name)
            .field("symbol", &self.symbol)
            .field("decimals", &self.decimals)
            // TODO: implement token debug formatter
            // .field("token", &self.token)
            .finish()
    }
}

#[integration_tests_bindgen(internal)]
#[near_bindgen]
impl TokenContract {
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        Self {
            name,
            symbol,
            decimals,
            token: FungibleToken::new(b"t".to_vec()),
        }
    }

    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_deposit(&account_id, amount.into());
    }

    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_withdraw(&account_id, amount.into());
    }
}

#[integration_tests_bindgen(internal)]
#[near_bindgen]
impl FungibleTokenMetadataProvider for TokenContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: self.decimals,
        }
    }
}

#[integration_tests_bindgen(internal)]
#[near_bindgen]
impl StorageManagement for TokenContract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.token.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenCore for TokenContract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }
    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }
    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[integration_tests_bindgen(internal)]
#[near_bindgen]
impl FungibleTokenResolver for TokenContract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {}
        used_amount.into()
    }
}

// Implementations of next four functions below is needed because currently
// #[integration_tests_bindgen] macro does not support PromiseOrValue that is returned by ft_transfer_call
// as it does not support Deserialize trait.
// Can be removed once it is supported and changed to #[integration_tests_bindgen] macro for impl block above.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
impl TokenContractTest {
    pub async fn ft_transfer(
        &self,
        receiver_id: workspaces::AccountId,
        amount: U128,
        memo: Option<String>,
        caller: &workspaces::Account,
        attached_deposit: u128,
    ) -> integration_tests_toolset::error::Result<integration_tests_toolset::tx_result::TxResult<()>>
    {
        use integration_tests_toolset::pending_tx::call::Call;
        use integration_tests_toolset::{res_logger::ResLogger, tx_result::FromRes};
        let args = near_sdk::serde_json::json!({
          "receiver_id":receiver_id,"amount":amount,"memo":memo
        })
        .to_string()
        .into_bytes();
        let storage_usage_before = if self.measure_storage_usage {
            self.contract.view_account().await?.storage_usage
        } else {
            0
        };
        let res = integration_tests_toolset::pending_tx::payable_tx::PayablePendingTx::new(
            &self.contract,
            String::from("ft_transfer"),
            args,
            attached_deposit,
        )
        .call(caller)
        .await?;
        let storage_usage = if self.measure_storage_usage {
            Some(
                self.contract.view_account().await?.storage_usage as i64
                    - storage_usage_before as i64,
            )
        } else {
            None
        };
        res.check_res_log_failures()?;
        integration_tests_toolset::tx_result::CallResult::from_res(
            "ft_transfer".to_owned(),
            (),
            storage_usage,
            res,
        )
    }

    pub async fn ft_transfer_call(
        &self,
        receiver_id: workspaces::AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
        caller: &workspaces::Account,
        attached_deposit: u128,
    ) -> integration_tests_toolset::error::Result<
        integration_tests_toolset::tx_result::TxResult<Option<U128>>,
    > {
        use integration_tests_toolset::pending_tx::call::Call;
        use integration_tests_toolset::{res_logger::ResLogger, tx_result::FromRes};
        let args = near_sdk::serde_json::json!({
          "receiver_id":receiver_id,"amount":amount,"memo":memo,"msg":msg
        })
        .to_string()
        .into_bytes();
        let storage_usage_before = if self.measure_storage_usage {
            self.contract.view_account().await?.storage_usage
        } else {
            0
        };
        let res = integration_tests_toolset::pending_tx::payable_tx::PayablePendingTx::new(
            &self.contract,
            String::from("ft_transfer_call"),
            args,
            attached_deposit,
        )
        .call(caller)
        .await?;
        let storage_usage = if self.measure_storage_usage {
            Some(
                self.contract.view_account().await?.storage_usage as i64
                    - storage_usage_before as i64,
            )
        } else {
            None
        };
        res.check_res_log_failures()?;

        // I gave up here. I don't know how to convert ExecutionResult<Value>
        // to something, that can be recognized as Promise or Value and obtain value from there
        // Maybe next snippets will help someone to finish this
        // let real_value = res.clone().into_result()?;
        // let value = real_value.maybe_to_value();

        integration_tests_toolset::tx_result::CallResult::from_res(
            "ft_transfer_call".to_owned(),
            None,
            storage_usage,
            res,
        )
    }
    pub async fn ft_total_supply(
        &self,
    ) -> integration_tests_toolset::error::Result<
        integration_tests_toolset::tx_result::TxResult<U128>,
    > {
        use integration_tests_toolset::pending_tx::view::View;
        use integration_tests_toolset::{res_logger::ResLogger, tx_result::FromRes};
        let args = near_sdk::serde_json::json!({}).to_string().into_bytes();
        let storage_usage_before = if self.measure_storage_usage {
            self.contract.view_account().await?.storage_usage
        } else {
            0
        };
        let res = integration_tests_toolset::pending_tx::immutable_tx::ImmutablePendingTx::new(
            &self.contract,
            String::from("ft_total_supply"),
            args,
        )
        .view()
        .await?;
        let storage_usage = if self.measure_storage_usage {
            Some(
                self.contract.view_account().await?.storage_usage as i64
                    - storage_usage_before as i64,
            )
        } else {
            None
        };
        res.check_res_log_failures()?;
        integration_tests_toolset::tx_result::view_result::ViewResult::from_res(
            "ft_total_supply".to_owned(),
            integration_tests_toolset::tx_result::view_result::ViewResult::value_from_res(&res)?,
            storage_usage,
            res,
        )
    }
    pub async fn ft_balance_of(
        &self,
        account_id: workspaces::AccountId,
    ) -> integration_tests_toolset::error::Result<
        integration_tests_toolset::tx_result::TxResult<U128>,
    > {
        use integration_tests_toolset::pending_tx::view::View;
        use integration_tests_toolset::{res_logger::ResLogger, tx_result::FromRes};
        let args = near_sdk::serde_json::json!({ "account_id": account_id })
            .to_string()
            .into_bytes();
        let storage_usage_before = if self.measure_storage_usage {
            self.contract.view_account().await?.storage_usage
        } else {
            0
        };
        let res = integration_tests_toolset::pending_tx::immutable_tx::ImmutablePendingTx::new(
            &self.contract,
            String::from("ft_balance_of"),
            args,
        )
        .view()
        .await?;
        let storage_usage = if self.measure_storage_usage {
            Some(
                self.contract.view_account().await?.storage_usage as i64
                    - storage_usage_before as i64,
            )
        } else {
            None
        };
        res.check_res_log_failures()?;
        integration_tests_toolset::tx_result::view_result::ViewResult::from_res(
            "ft_balance_of".to_owned(),
            integration_tests_toolset::tx_result::view_result::ViewResult::value_from_res(&res)?,
            storage_usage,
            res,
        )
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = TokenContract::new("token".to_string(), "TKN".to_string(), 12);
        testing_env!(context
            .attached_deposit(125 * env::storage_byte_cost())
            .build());
        contract.storage_deposit(Some(accounts(0)), None);
        contract.mint(accounts(0), 1_000_000.into());
        assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());

        testing_env!(context
            .attached_deposit(125 * env::storage_byte_cost())
            .build());
        contract.storage_deposit(Some(accounts(1)), None);
        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.ft_transfer(accounts(1), 1_000.into(), None);
        assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());

        contract.burn(accounts(1), 500.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());
    }
}
