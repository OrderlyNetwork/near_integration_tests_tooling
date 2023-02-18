use std::collections::HashMap;

use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use workspaces::{types::Balance, Account, AccountId, Contract};

pub trait ContractController {
    type ContractTemplate;
    fn get_template(&self) -> &Self::ContractTemplate;
    fn get_contract(&self) -> &Contract;
}

#[async_trait]
pub trait ContractInitializer<ContractTemplate> {
    fn get_id(&self) -> AccountId;
    fn get_wasm(&self) -> Vec<u8>;
    fn get_storage_deposit_amount(&self) -> Balance;
    fn get_role_accounts(&self) -> HashMap<String, AccountId>;
    async fn initialize_contract_template(
        &self,
        contract: Contract,
        roles: HashMap<String, Account>,
    ) -> Result<Box<dyn ContractController<ContractTemplate = ContractTemplate>>, TestError>;
}
