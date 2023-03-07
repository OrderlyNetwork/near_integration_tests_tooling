use integration_tests_toolset::{
    error::{Result, TestError},
    tx_result::TxResult,
};
use near_sdk::json_types::U128;
use owo_colors::{AnsiColors, OwoColorize};
use rand::Rng;
use test_contract::TestContractTest;
use test_token::TokenContractTest;
use workspaces::Account;

async fn _ft_transfer_call_with_storage_measure(
    token: &TokenContractTest,
    contract_template: &TestContractTest,
    amount: u128,
    msg: String,
    issuer: &Account,
) -> Result<TxResult<Option<U128>>> {
    let storage_usage_before = contract_template
        .contract
        .view_account()
        .await?
        .storage_usage;
    let mut res = token
        .ft_transfer_call(
            contract_template.contract.as_account().id().clone(),
            U128(amount),
            None,
            msg,
            &issuer,
            1,
        )
        .await?;
    let storage_usage_after = contract_template
        .contract
        .view_account()
        .await?
        .storage_usage;
    res.storage_usage = Some(storage_usage_after as i64 - storage_usage_before as i64);
    Ok(res)
}

pub async fn error_operation() -> Result<()> {
    Err(TestError::Custom("This operation always fails".to_owned()))
}

pub async fn sleep_operation(ms: u64) -> Result<()> {
    Ok(tokio::time::sleep(std::time::Duration::from_millis(ms)).await)
}

pub async fn numbered_operation(number: u64, color: AnsiColors) -> Result<()> {
    let sleep_duration = rand::thread_rng().gen_range(1..10);
    tokio::time::sleep(std::time::Duration::from_millis(sleep_duration)).await;
    println!(
        "My number is {}, I slept for {} ms",
        number.color(color),
        sleep_duration
    );
    Ok(())
}
