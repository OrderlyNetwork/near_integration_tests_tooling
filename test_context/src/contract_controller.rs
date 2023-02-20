use crate::common::TestAccount;
use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use std::any::Any;
use std::collections::HashMap;
use workspaces::{types::Balance, Account, AccountId, Contract};

pub trait ControllerAsAny {
    type DowncastType;
    fn get_type(&self) -> &Self::DowncastType;
    fn as_any(&self) -> &dyn Any;
}

pub trait ContractController: ControllerAsAny + Send + Sync {
    type ContractTemplate;
    fn get_template(&self) -> &Self::ContractTemplate;
    fn get_contract(&self) -> &Contract;
}

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
    ) -> Result<
        Box<dyn ContractController<ContractTemplate = ContractTemplate, DowncastType = T>>,
        TestError,
    >;
}
