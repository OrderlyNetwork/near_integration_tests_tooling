use anyhow::Ok;
use near_units::parse_near;
use test_contract::TestContractTest;

#[tokio::test]
async fn integration_test_example() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let contract = worker
        .dev_deploy(include_bytes!("../../res/test_contract.wasm"))
        .await?;

    let user = worker.dev_create_account().await?;

    let contract_template = TestContractTest {
        contract,
        measure_storage_usage: true,
    };

    contract_template
        .new(10, &contract_template.contract.as_account(), 1u128)
        .await?;

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    // repeated init should fail
    let res = contract_template
        .new(11, &contract_template.contract.as_account(), 1u128)
        .await;

    assert!(res.is_err());

    assert_eq!(
        contract_template
            .view_param_account_id_ret_account_id(user.id().clone().to_string().parse().unwrap())
            .await?
            .value,
        user.id().to_string().parse().unwrap()
    );

    assert_eq!(
        contract_template
            .view_param_vec_account_id_ret_vec_account_id(vec![user
                .id()
                .to_string()
                .parse()
                .unwrap()])
            .await?
            .value,
        vec![user.id().to_string().parse().unwrap()]
    );

    contract_template.migrate_state(&user).await?;

    let res = contract_template.call_no_param_ret_u64(&user).await?;

    assert_eq!(res.value, 1);

    assert_eq!(contract_template.view_no_param_ret_u64().await?.value, 1);

    contract_template
        .call_no_param_no_ret_payable(&user, parse_near!("1 yN"))
        .await?;

    assert_eq!(contract_template.view_no_param_ret_u64().await?.value, 2);

    let res = contract_template
        .call_param_u64_ret_u64_handle_res(2, &user)
        .await?;

    assert_eq!(res.value, 4);

    let res = contract_template.view_no_param_ret_u64().await?;

    assert_eq!(res.value, 4);

    let res = contract_template.view_no_param_ret_error_handle_res().await;

    assert!(res.is_err());

    let res = contract_template.view_no_param_ret_error_handle_res().await;

    let res = res.unwrap_err();
    println!("res: {}", res);

    let res = contract_template
        .call_no_param_ret_error_handle_res(&user)
        .await;

    assert!(res.is_err());
    let res = res.unwrap_err();
    println!("res: {}", res);

    Ok(())
}
