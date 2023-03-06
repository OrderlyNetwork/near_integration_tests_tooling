use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use maplit::hashmap;
use std::collections::HashMap;
use test_context::{
    common::TestAccount, contract_initializer::ContractInitializer, token_info::eth,
};
use test_contract::TestContractTest;
use workspaces::AccountId;

pub struct Initializer {}

#[async_trait]
impl ContractInitializer<TestContractTest, ContractHolder> for Initializer {
    fn get_id(&self) -> AccountId {
        "any_acc.test.near".parse().unwrap()
    }

    fn get_wasm(&self) -> Vec<u8> {
        include_bytes!("../../res/test_contract.wasm").to_vec()
    }

    fn get_storage_deposit_amount(&self) -> workspaces::types::Balance {
        100_000_000_0000
    }

    fn get_role_accounts(&self) -> HashMap<String, TestAccount> {
        hashmap! {
            "owner".to_string() => TestAccount { account_id: "owner.test.near".parse().unwrap(), mint_amount: hashmap! {
                eth().to_string() => eth().parse("15").unwrap(),
            },}
        }
    }

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
