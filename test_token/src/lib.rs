// In case of any changes in this test TokenContract the build.sh script should be used in order to update
// the related wasm file in the wasm target folder

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
    env,
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
};

#[integration_tests_bindgen]
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

/// This is test implementation of the core fungible token logic.
/// It also generate test contract for integration tests.
#[integration_tests_bindgen]
#[near_bindgen]
impl TokenContract {
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8, initial_mint: Option<U128>) -> Self {
        let mut this = Self {
            name,
            symbol,
            decimals,
            token: FungibleToken::new(b"t".to_vec()),
        };

        let current_id = env::current_account_id();

        this.token.internal_register_account(&current_id);

        let initial_mint = initial_mint.unwrap_or(0.into());

        if initial_mint.0 > 0 {
            this.token.internal_deposit(&current_id, initial_mint.0);
            near_contract_standards::fungible_token::events::FtMint {
                owner_id: &current_id,
                amount: &initial_mint,
                memo: Some("Initial tokens supply is minted"),
            }
            .emit();
        }

        this
    }

    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_deposit(&account_id, amount.into());
    }

    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_withdraw(&account_id, amount.into());
    }
}

#[integration_tests_bindgen]
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

#[integration_tests_bindgen]
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

#[integration_tests_bindgen]
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

#[integration_tests_bindgen]
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

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract =
            TokenContract::new("token".to_string(), "TKN".to_string(), 12, Some(0.into()));
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
