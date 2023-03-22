use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use maplit::hashmap;
use scenario_toolset::{
    contract_initializer::ContractInitializer,
    utils::{token_info::eth, TestAccount},
};
use std::collections::HashMap;
use test_contract::TestContractTest;
use workspaces::AccountId;

pub struct Initializer {}

/// This structure and trait should be implemented in target project to implement
/// specific initialization logic for particular contract
/// Contract initializer that initialize contract template and owner role account
/// Later owner role account can be obtaing from contract_holder and used in tests
/// Contract initializer put as parameter to initialize_context function
#[async_trait]
impl ContractInitializer<TestContractTest, ContractHolder> for Initializer {
    /// Provide contract id for initialize_context
    fn get_id(&self) -> AccountId {
        "any_acc.test.near".parse().unwrap()
    }

    /// Provide contract wasm for initialize_context
    fn get_wasm(&self) -> Vec<u8> {
        include_bytes!("../../res/test_contract.wasm").to_vec()
    }

    /// Provide role accounts for initialize_context, that create role accounts,
    /// mint tokens for them and provide them to initialize_contract_template
    fn get_role_accounts(&self) -> HashMap<String, TestAccount> {
        hashmap! {
            "owner".to_string() => TestAccount { account_id: "owner.test.near".parse().unwrap(), mint_amount: hashmap! {
                eth().to_string() => eth().parse("15").unwrap(),
            },}
        }
    }

    /// Initialize contract template using role accounts
    async fn initialize_contract_template(
        &self,
        contract: workspaces::Contract,
        roles: HashMap<String, workspaces::Account>,
    ) -> Result<(TestContractTest, ContractHolder), TestError> {
        let contract_id = contract.as_account().clone();
        let contract_template = TestContractTest {
            contract,
            measure_storage_usage: true,
        };

        contract_template.new(10, &contract_id, 1u128).await?;
        let owner = roles.get("owner").unwrap().clone();

        Ok((contract_template, ContractHolder { owner }))
    }
}

#[allow(dead_code)]
pub struct ContractHolder {
    pub owner: workspaces::Account,
}
