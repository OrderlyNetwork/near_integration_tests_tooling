mod contract_initializer;

use crate::contract_initializer::{ContractHolder, Initializer};
use integration_tests_toolset::error::TestError;
use near_units::parse_near;
use scenario_toolset::{
    context_initialize::initialize_context,
    utils::{
        account_id,
        token_info::{wnear, TokenInfo},
        TestAccount,
    },
};
use std::collections::HashMap;
use test_contract::TestContractTest;
use workspaces::types::Balance;

/// Minimal example of contract initializer usage
#[tokio::test]
async fn test_initializer_usage() -> anyhow::Result<()> {
    let (_, contract_template, _, _, _) = initialize_context(&[], &[], &Initializer {}).await?;

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    Ok(())
}

/// Example of massive context initialization
/// This test initialize 10 tokens and 10 accounts with minting each token to each account
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
