mod contract_initializer;
mod operation_examples;

use crate::contract_initializer::Initializer;
use futures::FutureExt;
use integration_tests_toolset::{
    error::TestError,
    statistic::{
        call_counter::CallCounter,
        gas_usage_aggregator::GasUsage,
        statistic_consumer::{Statistic, StatisticConsumer},
        statistic_group_ext::StatisticGroupExt,
        statistic_group_printer::StatisticGroupPrinter,
        storage_usage_aggregator::StorageUsage,
    },
    tx_result::{IntoMutRefs, TxResult},
};
use maplit::hashmap;
use operation_examples::{error_operation, numbered_operation, sleep_operation};
use owo_colors::AnsiColors;
use scenario_toolset::{
    batch::{make_op, make_unit_op, Batch},
    context_initialize::initialize_context,
    utils::{
        maker_id,
        token_info::{eth, usdc},
        TestAccount,
    },
};
use std::pin::Pin;
use test_contract::TestContractTest;
use workspaces::AccountId;

/// Example of minimum batch usage
/// Initialize context and run batch with one operation
#[tokio::test]
async fn test_minimal_batch() -> anyhow::Result<()> {
    let (_, contract_template, _, _, _) = initialize_context(&[], &[], &Initializer {}).await?;
    Batch::new()
        .add_chain_op(make_op(contract_template.view_no_param_ret_u64()))
        .run()
        .await?;

    Ok(())
}

/// Example of butch usage with custom operation that panics
#[tokio::test]
#[should_panic = "This operation always fails"]
async fn test_chain_catch_error() {
    initialize_context(&[], &[], &Initializer {}).await.unwrap();

    Batch::new()
        .add_chain_op(make_unit_op(error_operation()))
        .run()
        .await
        .unwrap();
}

/// Example of butch usage with contract operation that panics
#[tokio::test]
#[should_panic = "View function rised error!"]
async fn test_concurrent_catch_error() {
    let (_, contract_template, _, _, _) =
        initialize_context(&[], &[], &Initializer {}).await.unwrap();

    Batch::new()
        .add_concurrent_op(make_op(
            contract_template.view_no_param_ret_error_handle_res(),
        ))
        .run()
        .await
        .unwrap();
}

/// Example of mixed sequential and concurrent operations
#[tokio::test]
async fn test_chain_execution() -> anyhow::Result<()> {
    initialize_context(&[], &[], &Initializer {}).await.unwrap();

    Batch::new()
        .add_chain_ops(
            (0..100)
                .map(|i| make_unit_op(numbered_operation(i, AnsiColors::Green)))
                .collect::<Vec<_>>(),
        )
        .add_concurrent_ops(
            (0..100)
                .map(|i| make_unit_op(numbered_operation(i, AnsiColors::Blue)))
                .collect::<Vec<_>>(),
        )
        .run()
        .await?;

    Ok(())
}

/// Example of different contract operations in batch with statistic processing
#[tokio::test]
async fn test_batch_combination() -> anyhow::Result<()> {
    let accounts = [TestAccount {
        account_id: maker_id(),
        mint_amount: hashmap! {},
    }];

    let (_, contract_template, _, _, accounts) =
        initialize_context(&[], &accounts, &Initializer {}).await?;

    let stat = Batch::new()
        .add_chain_ops(vec![
            make_op(contract_template.view_account_id(accounts[0].id().clone())),
            make_op(contract_template.call_no_param_no_ret_payable(&accounts[0], 1)),
            make_unit_op(sleep_operation(1)),
            make_op(contract_template.view_no_param_ret_u64()),
            make_op(contract_template.call_no_param_ret_u64(&accounts[0])),
            make_op(contract_template.view_option_account_id(Some(accounts[0].id().clone()))),
            make_op(contract_template.call_param_u64_ret_u64_handle_res(1, &accounts[0])),
            make_op(
                contract_template.view_param_account_id_ret_account_id(accounts[0].id().clone()),
            ),
        ])
        .add_concurrent_ops(
            (0..100)
                .map(|i| {
                    Batch::new()
                        .add_chain_op(make_unit_op(numbered_operation(i, AnsiColors::Green)))
                        .add_chain_op(make_op(contract_template.view_no_param_ret_u64()))
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .run()
        .await?
        .process_statistic([
            Box::new(GasUsage::default()),
            Box::new(StorageUsage::default()),
            Box::new(CallCounter::default()),
        ]);

    println!("{}", stat);

    Ok(())
}

/// Example of different kinds of batch operations with statistic processing and printing
#[tokio::test]
async fn block_operations_example() -> anyhow::Result<()> {
    let (_, contract_template, _, [_eth, _usdc], [maker_account]) = initialize_context(
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

    // Create statistic consumers separately to be able to use them in different futures
    let mut statistic_consumer: [Box<dyn StatisticConsumer>; 1] = [Box::new(GasUsage::default())];

    // This operation populate statistic to statistic_consumer itself
    // It can be used to inject statistic for custom operations
    let future_that_populates_statistic_itself = contract_template
        .call_no_param_ret_u64(&maker_account)
        .map(|res| {
            res.map(|tx| {
                tx.populate_statistic(&mut statistic_consumer.into_refs());
                42
            })
        });

    // Example of custom operation with params, that calls contract function
    fn future_with_params<'a>(
        template: &'a TestContractTest,
        maker_account_id: AccountId,
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<TxResult<workspaces::AccountId>, TestError>>
                + std::marker::Send
                + 'a,
        >,
    > {
        template.view_account_id(maker_account_id).boxed()
    }

    // Example of future created from closure
    let future_from_closure = || {
        contract_template
            .call_no_param_ret_u64(&maker_account)
            .map(|res| res.map(|tx| Statistic::from(tx)))
            .boxed()
    };

    // Example of immediate future created from contract function
    let future_from_contract_template_function = contract_template
        .view_account_id(maker_account.id().clone())
        .boxed()
        .into();

    // Create batch with different kinds of operations
    let block1 = Batch::new()
        .add_concurrent_ops(vec![
            make_unit_op(future_that_populates_statistic_itself),
            make_op(contract_template.view_account_id(maker_account.id().clone())),
            make_op(future_with_params(
                &contract_template,
                maker_account.id().clone(),
            )),
        ])
        .add_chain_op(make_op(
            contract_template.view_account_id(maker_account.id().clone()),
        ))
        .add_chain_op(
            Batch::new()
                .add_chain_op(future_from_closure().into())
                .add_concurrent_op(future_from_contract_template_function)
                .into(),
        );

    block1
        .run()
        .await?
        .populate_statistic(statistic_consumer)
        .print_statistic()?;

    Ok(())
}
