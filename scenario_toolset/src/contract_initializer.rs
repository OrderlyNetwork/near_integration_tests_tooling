use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use std::collections::HashMap;
use workspaces::{types::Balance, Account, AccountId, Contract};

use crate::utils::TestAccount;

#[async_trait]
pub trait ContractInitializer<ContractTemplate, T> {
    fn get_id(&self) -> AccountId;
    fn get_wasm(&self) -> Vec<u8>;
    fn get_storage_deposit_amount(&self) -> Balance;
    fn get_role_accounts(&self) -> HashMap<String, TestAccount>;
    async fn initialize_contract_template(
        &self,
        contract: Contract,
        roles: HashMap<String, Account>,
    ) -> Result<(ContractTemplate, T), TestError>;
}
