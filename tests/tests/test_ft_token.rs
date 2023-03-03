mod contract_initializer;

use crate::contract_initializer::Initializer;
use maplit::hashmap;
use near_units::parse_near;
use test_context::{
    common::{maker_id, TestAccount},
    context::initialize_context,
    token_info::{eth, usdc},
};

#[tokio::test]
async fn test_ft_transfer_usage() -> anyhow::Result<()> {
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

    let _owner = &contract_holder.owner;

    eth.storage_deposit(
        None,
        None,
        contract_template.contract.as_account(),
        parse_near!("1N"),
    )
    .await?;

    eth.storage_deposit(None, None, eth.contract.as_account(), parse_near!("1N"))
        .await?;

    eth.mint(
        eth.contract.id().clone(),
        10.into(),
        eth.contract.as_account(),
        1u128,
    )
    .await?;

    eth.ft_transfer(
        contract_template.contract.id().clone(),
        10.into(),
        None,
        eth.contract.as_account(),
        1u128,
    )
    .await?;

    assert_eq!(
        eth.ft_balance_of(contract_template.contract.id().clone())
            .await?
            .value
            .0,
        10
    );

    eth.ft_transfer_call(
        contract_template.contract.id().clone(),
        10.into(),
        None,
        "Get my money!".to_owned(),
        &maker_account,
        1u128,
    )
    .await?;

    assert_eq!(
        eth.ft_balance_of(contract_template.contract.id().clone())
            .await?
            .value
            .0,
        20
    );

    Ok(())
}
