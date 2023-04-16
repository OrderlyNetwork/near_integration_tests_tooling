mod contract_initializer;

use integration_tests_toolset::statistic::{
    gas_usage_aggregator::GasUsage,
    statistic_consumer::{Statistic, StatisticConsumer},
    storage_usage_aggregator::StorageUsage,
};
use maplit::hashmap;
use near_units::parse_near;
use scenario_toolset::{
    context_initialize::initialize_context,
    utils::{
        maker_id,
        token_info::{eth, usdc},
        TestAccount,
    },
};
use serde_json::json;
use workspaces::types::{KeyType, SecretKey};

use crate::contract_initializer::Initializer;

// Test for fungible token test contract
#[tokio::test]
async fn test_ft_transfer_usage() -> anyhow::Result<()> {
    // Initialize context with contract template and two tokens; Mint tokens for maker account
    let (_, contract_template, contract_holder, [eth, _usdc], [maker_account]) =
        initialize_context(
            &[eth(), usdc()],
            &[TestAccount {
                account_id: maker_id(),
                mint_amount: hashmap! {
                    eth().to_string() => eth().parse("15")?
                },
            }],
            &Initializer {},
        )
        .await?;

    // This is how contract's role accounts can be obtained from contract_holder
    let _owner = &contract_holder.owner;

    // Transfer tokens to contract template
    eth.ft_transfer(
        contract_template.contract.id().clone(),
        10.into(),
        None,
        eth.contract.as_account(),
        1u128,
    )
    .await?;

    // Check balance of contract template
    assert_eq!(
        eth.ft_balance_of(contract_template.contract.id().clone())
            .await?
            .value
            .0,
        10
    );

    // Check ft_transfer_call method
    eth.ft_transfer_call(
        contract_template.contract.id().clone(),
        10.into(),
        None,
        "Get my money!".to_owned(),
        &maker_account,
        1u128,
    )
    .await?;

    // Check that tokens were transferred to contract template
    assert_eq!(
        eth.ft_balance_of(contract_template.contract.id().clone())
            .await?
            .value
            .0,
        20
    );

    Ok(())
}

#[tokio::test]
async fn measure_token_deposit() -> anyhow::Result<()> {
    // Initialize context with contract template and eth token
    let (_, contract_template, _, [eth], [account1, account2]) = initialize_context(
        &[eth()],
        &[
            TestAccount {
                account_id: "account1.test.near".parse().unwrap(),
                mint_amount: hashmap! {
                    eth().to_string() => eth().parse("15")?
                },
            },
            TestAccount {
                account_id: "account2.test.near".parse().unwrap(),
                mint_amount: hashmap! {
                    eth().to_string() => eth().parse("15")?
                },
            },
        ],
        &Initializer {},
    )
    .await?;

    // define consumers
    let mut gas_stat: Box<dyn StatisticConsumer> = Box::new(GasUsage::default());
    let mut storage_stat: Box<dyn StatisticConsumer> = Box::new(StorageUsage::default());

    // make a native deposit call
    contract_template
        .deposit_native_token(&account1, parse_near!("1 N"))
        .await?
        .populate_statistic(&mut [&mut gas_stat, &mut storage_stat]);

    let storage_usage_before = contract_template
        .contract
        .view_account()
        .await?
        .storage_usage;

    // Transfer tokens to contract template
    eth.ft_transfer_call(
        contract_template.contract.id().clone(),
        10.into(),
        None,
        String::from(""),
        &account2,
        1u128,
    )
    .await?
    .populate_statistic(&mut [&mut gas_stat]);

    // Here we made a work around in order to populate the storage usage of our contract
    let mut statistic = Statistic::default();
    statistic.storage_usage = Some(
        contract_template
            .contract
            .view_account()
            .await?
            .storage_usage as i64
            - storage_usage_before as i64,
    );
    statistic.func_name = String::from("ft_on_transfer");

    storage_stat.consume_statistic(&statistic);

    gas_stat.print_statistic()?;
    storage_stat.print_statistic()?;

    Ok(())
}

// Worth to mention that easy to miss and write wrong test because of passing wrong type, missing deposit, using wrong parameter name etc.
// With generated bindings all that could be covered with static checks
#[tokio::test]
async fn measure_token_deposit_original() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let contract_wasm = include_bytes!("../../res/test_contract.wasm").to_vec();

    let contract = worker
        .create_tla_and_deploy(
            "contract.test.near".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
            &contract_wasm,
        )
        .await?
        .into_result()?;

    let token_wasm = include_bytes!("../../res/test_token.wasm").to_vec();

    let token_contract = worker
        .create_tla_and_deploy(
            "token1.test.near".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
            &token_wasm,
        )
        .await?
        .into_result()?;

    let user1 = worker
        .create_tla(
            "user1.test.near".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
        )
        .await?
        .into_result()?;

    let user2 = worker
        .create_tla(
            "user2.test.near".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
        )
        .await?
        .into_result()?;

    let res = contract
        .call("new")
        .args_json(json!({
          "initial_state":0
        }))
        .deposit(parse_near!("1yN"))
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());

    let res = token_contract
        .call("new")
        .args_json(json!({
            "name": String::from("TOKEN1"),"symbol": "TKN1","decimals":6,"initial_mint":String::from("1000000000000")
          }))
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());

    let res = contract
        .as_account()
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": Option::<near_sdk::AccountId>::None,
            "registration_only" : Option::<bool>::None
        }))
        .max_gas()
        .deposit(parse_near!("0.1N"))
        .transact()
        .await?;

    assert!(res.is_success());

    let res = user2
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": Option::<near_sdk::AccountId>::None,
            "registration_only" : Option::<bool>::None
        }))
        .max_gas()
        .deposit(parse_near!("0.1N"))
        .transact()
        .await?;

    assert!(res.is_success());

    let storage_usage_before = contract.view_account().await?.storage_usage;

    let res = user1
        .call(contract.id(), "deposit_native_token")
        .deposit(parse_near!("10N"))
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());

    println!(
        "deposit_native_token gas usage = {:.5}Near storage usage = {}Near",
        res.total_gas_burnt as f64 / 10_000_000_000_000_000i64 as f64,
        (contract.view_account().await?.storage_usage - storage_usage_before) as f64
            / (near_sdk::ONE_NEAR.saturating_div(near_sdk::env::storage_byte_cost()) as f64)
    );

    let res = token_contract
        .call("mint")
        .args_json(json!({
            "account_id": user2.id(), "amount": String::from("10")
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());

    let storage_usage_before = contract.view_account().await?.storage_usage;

    let res = user2
        .call(token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": contract.id(),
            "amount": String::from("10"),
            "memo": Option::<String>::None,
            "msg": String::from(""),
        }))
        .deposit(parse_near!("1yN"))
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());

    println!(
        "ft_transfer_call gas usage = {:.5}Near storage usage = {}Near",
        res.total_gas_burnt as f64 / 10_000_000_000_000_000i64 as f64,
        (contract.view_account().await?.storage_usage - storage_usage_before) as f64
            / (near_sdk::ONE_NEAR.saturating_div(near_sdk::env::storage_byte_cost()) as f64)
    );

    Ok(())
}
