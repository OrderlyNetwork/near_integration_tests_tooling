use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use std::collections::HashMap;
use workspaces::{Account, AccountId, Contract};

use crate::utils::TestAccount;

/// Contract initializer trait
/// These functions called during context initialization
/// and used to initialize contract template and role accounts
#[async_trait]
pub trait ContractInitializer<ContractTemplate, T> {
    /// Provide contract id for initialize_context
    fn get_id(&self) -> AccountId;
    /// Provide contract wasm for initialize_context
    fn get_wasm(&self) -> Vec<u8>;
    /// Provide role accounts for initialize_context, that create role accounts,
    /// mint tokens for them and provide them to initialize_contract_template
    fn get_role_accounts(&self) -> HashMap<String, TestAccount>;
    /// Initialize contract template using role accounts
    async fn initialize_contract_template(
        &self,
        contract: Contract,
        roles: HashMap<String, Account>,
    ) -> Result<(ContractTemplate, T), TestError>;
}
