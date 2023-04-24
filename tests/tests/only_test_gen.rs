use integration_tests_toolset::{
    statistic::{
        gas_usage_aggregator::GasUsage, statistic_consumer::StatisticConsumer,
        statistic_group_printer::StatisticGroupPrinter, storage_usage_aggregator::StorageUsage,
    },
    tx_result::IntoMutRefs,
};
use near_units::parse_near;
use test_contract::TestContractTest;

/// Test of bindgen macro generated functions
/// This is mostly for internal testing of macro generated functions
/// However it shows that standalone usage of macro generated functions
/// can use statistic consumers and obtain statistic
#[tokio::test]
async fn standalone_test_gen_functions() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let contract = worker
        .dev_deploy(include_bytes!("../../res/test_contract.wasm"))
        .await?;

    let user = worker.dev_create_account().await?;

    let contract_template = TestContractTest {
        contract,
        measure_storage_usage: true,
    };

    let mut statistic_consumers: [Box<dyn StatisticConsumer>; 2] = [
        Box::new(GasUsage::default()),
        Box::new(StorageUsage::default()),
    ];
    let consumers_refs = &mut statistic_consumers.into_refs();

    contract_template
        .new(10, &contract_template.contract.as_account(), 1u128)
        .await?
        .populate_statistic(consumers_refs);

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    // repeated init should fail
    let res = contract_template
        .new(11, &contract_template.contract.as_account(), 1u128)
        .await;

    assert!(res.is_err());

    // populate_statistic returns TxResult, so it can be used in chain with
    // obtaining resulting call value
    assert_eq!(
        contract_template
            .view_param_account_id_ret_account_id(user.id().clone())
            .await?
            .populate_statistic(consumers_refs)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_vec_tuple_with_account_id(vec![(user.id().clone(), 1)])
            .await?
            .populate_statistic(consumers_refs)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_arr_tuples_with_account_id([(user.id().clone(), 1)])
            .await?
            .populate_statistic(consumers_refs)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_vec_tuple_of_vec_tuples_with_account_id(vec![(
                vec![(user.id().clone(), 1)],
                1
            )])
            .await?
            .populate_statistic(consumers_refs)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_vec_account_id_ret_vec_account_id(vec![user.id().clone()])
            .await?
            .populate_statistic(consumers_refs)
            .value,
        vec![user.id().clone()]
    );

    contract_template
        .migrate_state(&user)
        .await?
        .populate_statistic(consumers_refs);

    let res = contract_template
        .call_no_param_ret_u64(&user)
        .await?
        .populate_statistic(consumers_refs);

    assert_eq!(res.value, 1);

    assert_eq!(
        contract_template
            .view_no_param_ret_u64()
            .await?
            .populate_statistic(consumers_refs)
            .value,
        1
    );

    contract_template
        .call_no_param_no_ret_payable(&user, parse_near!("1 yN"))
        .await?
        .populate_statistic(consumers_refs);

    assert_eq!(
        contract_template
            .view_no_param_ret_u64()
            .await?
            .populate_statistic(consumers_refs)
            .value,
        2
    );

    let res = contract_template
        .call_param_u64_ret_u64_handle_res(2, &user)
        .await?
        .populate_statistic(consumers_refs);

    assert_eq!(res.populate_statistic(consumers_refs).value, 4);

    let res = contract_template
        .view_no_param_ret_u64()
        .await?
        .populate_statistic(consumers_refs);

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

    statistic_consumers.print_statistic()?;

    contract_template.view_account_id(user.id().clone()).await?;

    contract_template.view_ref_account_id(user.id()).await?;

    contract_template
        .view_option_account_id(Some(user.id().clone()))
        .await?;

    Ok(())
}
