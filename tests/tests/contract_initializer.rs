use async_trait::async_trait;
use integration_tests_toolset::error::TestError;
use maplit::hashmap;
use near_units::parse_near;
use std::collections::HashMap;
use test_context::{
    common::{account_id, TestAccount},
    context::initialize_context,
    contract_initializer::ContractInitializer,
    token_info::{eth, wnear, TokenInfo},
};
use test_contract::TestContractTest;
use workspaces::{types::Balance, AccountId};

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

#[tokio::test]
async fn test_initializer_usage() -> anyhow::Result<()> {
    let (_, contract_template, _, _, _) = initialize_context(&[], &[], &Initializer {}).await?;

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    Ok(())
}

#[tokio::test]
async fn test_massive_init() -> anyhow::Result<()> {
    let tokens = (0..10)
        .map(|i| TokenInfo {
            account_id: account_id("token", i),
            name: format!("token_{i}"),
            ticker: format!("TOKEN_{i}"),
            ..wnear()
        })
        .collect::<Vec<TokenInfo>>();

    let mint_amount = tokens
        .iter()
        .map(|token| (token.account_id.to_string(), parse_near!("1 N").into()))
        .collect::<HashMap<String, Balance>>();

    let accounts = (0..10)
        .map(|i| TestAccount {
            account_id: account_id("account", i),
            mint_amount: mint_amount.clone(),
        })
        .collect::<Vec<TestAccount>>();

    initialize_context::<TestContractTest, ContractHolder, 10, 10>(
        &tokens
            .try_into()
            .map_err(|_| TestError::Custom("Error tokens vector conversion".to_owned()))?,
        &accounts
            .try_into()
            .map_err(|_| TestError::Custom("Error accounts vector conversion".to_owned()))?,
        &Initializer {},
    )
    .await?;

    Ok(())
}
