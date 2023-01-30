use anyhow::Ok;
use integration_tests_toolset::{Call, View};
use near_units::parse_near;
use test_contract::TestContractTest;

#[tokio::test]
async fn integration_test_example() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let contract = worker
        .dev_deploy(include_bytes!("../../res/test_contract.wasm"))
        .await?;

    let user = worker.dev_create_account().await?;

    let contract_template = TestContractTest {};

    let res = contract_template.test_get_state().view(&contract).await?;

    assert_eq!(res.json::<u64>().unwrap(), 0);

    let res = contract_template
        .test_change_state()
        .call(&contract, &user)
        .await?;

    assert!(res.is_success());

    let res = contract_template.test_get_state().view(&contract).await?;

    assert_eq!(res.json::<u64>().unwrap(), 1);

    let res = contract_template
        .test_payable_change_state(parse_near!("1 yN"))
        .call(&contract, &user)
        .await?;

    assert!(res.is_success());

    let res = contract_template.test_get_state().view(&contract).await?;

    assert_eq!(res.json::<u64>().unwrap(), 2);

    Ok(())
}
