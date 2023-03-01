use async_trait::async_trait;
use futures::FutureExt;
use integration_tests_toolset::{
    error::TestError,
    statistic::{gas_usage_aggregator::GasUsage, statistic_consumer::Statistic},
};
use maplit::hashmap;
use near_units::parse_near;
use std::{any::Any, collections::HashMap, pin::Pin};
use test_context::{
    batch::{make_op, Batch},
    common::{maker_id, TestAccount},
    context::initialize_context,
    contract_controller::{ContractController, ContractInitializer, ControllerAsAny},
    token_info::{eth, usdc},
};
use test_contract::TestContractTest;
use workspaces::AccountId;

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

    let mut gas_usage = GasUsage::new();
    let mut statistic_consumers = [&mut gas_usage];

    contract_template
        .new(10, &contract_template.contract.as_account(), 1u128)
        .await?
        .populate_statistic(&mut statistic_consumers);

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    // repeated init should fail
    let res = contract_template
        .new(11, &contract_template.contract.as_account(), 1u128)
        .await;

    assert!(res.is_err());

    assert_eq!(
        contract_template
            .view_param_account_id_ret_account_id(user.id().clone())
            .await?
            .populate_statistic(&mut statistic_consumers)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_vec_tuple_with_account_id(vec![(user.id().clone(), 1)])
            .await?
            .populate_statistic(&mut statistic_consumers)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_arr_tuples_with_account_id([(user.id().clone(), 1)])
            .await?
            .populate_statistic(&mut statistic_consumers)
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
            .populate_statistic(&mut statistic_consumers)
            .value,
        user.id().clone()
    );

    assert_eq!(
        contract_template
            .view_param_vec_account_id_ret_vec_account_id(vec![user.id().clone()])
            .await?
            .populate_statistic(&mut statistic_consumers)
            .value,
        vec![user.id().clone()]
    );

    contract_template
        .migrate_state(&user)
        .await?
        .populate_statistic(&mut statistic_consumers);

    let res = contract_template
        .call_no_param_ret_u64(&user)
        .await?
        .populate_statistic(&mut statistic_consumers);

    assert_eq!(res.value, 1);

    assert_eq!(
        contract_template
            .view_no_param_ret_u64()
            .await?
            .populate_statistic(&mut statistic_consumers)
            .value,
        1
    );

    contract_template
        .call_no_param_no_ret_payable(&user, parse_near!("1 yN"))
        .await?
        .populate_statistic(&mut statistic_consumers);

    assert_eq!(
        contract_template
            .view_no_param_ret_u64()
            .await?
            .populate_statistic(&mut statistic_consumers)
            .value,
        2
    );

    let res = contract_template
        .call_param_u64_ret_u64_handle_res(2, &user)
        .await?
        .populate_statistic(&mut statistic_consumers);

    assert_eq!(res.populate_statistic(&mut statistic_consumers).value, 4);

    let res = contract_template
        .view_no_param_ret_u64()
        .await?
        .populate_statistic(&mut statistic_consumers);

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

    dbg!(gas_usage);

    contract_template.view_account_id(user.id().clone()).await?;

    contract_template.view_ref_account_id(user.id()).await?;

    contract_template
        .view_option_account_id(Some(user.id().clone()))
        .await?;

    Ok(())
}

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
    ) -> Result<
        Box<
            dyn test_context::contract_controller::ContractController<
                ContractTemplate = TestContractTest,
                DowncastType = ContractHolder,
            >,
        >,
        TestError,
    > {
        let contract_id = contract.as_account().clone();
        let contract_template = TestContractTest {
            contract,
            measure_storage_usage: true,
        };

        contract_template.new(10, &contract_id, 1u128).await?;
        let owner = roles.get("owner").unwrap().clone();

        Ok(Box::new(ContractHolder {
            contract: contract_template,
            owner,
        }))
    }
}

#[allow(dead_code)]
struct ContractHolder {
    contract: TestContractTest,
    pub owner: workspaces::Account,
}

impl ContractController for ContractHolder {
    type ContractTemplate = TestContractTest;

    fn get_template(&self) -> &Self::ContractTemplate {
        &self.contract
    }

    fn get_contract(&self) -> &workspaces::Contract {
        &self.contract.contract
    }
}

impl ControllerAsAny for ContractHolder {
    type DowncastType = ContractHolder;

    fn get_downcast_type(&self) -> &Self::DowncastType {
        self.as_any().downcast_ref::<Self::DowncastType>().unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[tokio::test]
async fn test_initializer_usage() -> anyhow::Result<()> {
    let (_, contract_controller, _, _) = initialize_context(&[], &[], &Initializer {}).await?;

    let contract_template = contract_controller.get_template();

    let res = contract_template.view_no_param_ret_u64().await?;
    assert_eq!(res.value, 10);

    Ok(())
}

#[tokio::test]
async fn test_ft_transfer_usage() -> anyhow::Result<()> {
    let (_, contract_controller, [eth, _usdc], [maker_account]) = initialize_context(
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

    // Downcast to ContractHolder to get access to it's fields (like contract and role_accounts)
    let contract_holder = contract_controller.get_downcast_type();

    let _owner = &contract_holder.owner;

    eth.storage_deposit(
        None,
        None,
        contract_controller.get_contract().as_account(),
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
        contract_controller.get_contract().id().clone(),
        10.into(),
        None,
        eth.contract.as_account(),
        1u128,
    )
    .await?;

    assert_eq!(
        eth.ft_balance_of(contract_controller.get_contract().id().clone())
            .await?
            .value
            .0,
        10
    );

    eth.ft_transfer_call(
        contract_controller.get_contract().id().clone(),
        10.into(),
        None,
        "Get my money!".to_owned(),
        &maker_account,
        1u128,
    )
    .await?;

    assert_eq!(
        eth.ft_balance_of(contract_controller.get_contract().id().clone())
            .await?
            .value
            .0,
        20
    );

    Ok(())
}

#[tokio::test]
async fn block_operations_test() -> anyhow::Result<()> {
    let (_, contract_controller, [_eth, _usdc], [maker_account]) = initialize_context(
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

    let mut statistic_consumer = GasUsage::new();

    let template = contract_controller.get_template();

    let fut = template
        .call_no_param_ret_u64(&maker_account)
        .map(|res| {
            res.map(|tx| {
                let tx = tx.populate_statistic(&mut [&mut statistic_consumer]);
                Statistic::from(tx)
            })
        })
        .boxed();

    fn f<'a>(
        template: &'a TestContractTest,
        maker_account_id: AccountId,
    ) -> Pin<Box<dyn futures::Future<Output = Result<Statistic, TestError>> + std::marker::Send + 'a>>
    {
        template
            .view_account_id(maker_account_id)
            .map(|res| res.map(|tx| tx.into()))
            .boxed()
    }

    let fut2 = || {
        template
            .call_no_param_ret_u64(&maker_account)
            .map(|res| res.map(|tx| Statistic::from(tx)))
            .boxed()
    };

    let fut3 = template
        .view_account_id(maker_account.id().clone())
        .boxed()
        .into();

    let block1 = Batch::new()
        .add_concurrent_ops(vec![
            fut.into(),
            make_op(template.view_account_id(maker_account.id().clone())),
            make_op(template.view_account_id(maker_account.id().clone())),
        ])
        .add_chain_op(make_op(
            template.view_account_id(maker_account.id().clone()),
        ))
        .add_chain_op(
            Batch::new()
                .add_chain_op(fut2().into())
                .add_concurrent_op(fut3)
                .into(),
        );

    let res = block1.run().await?;
    res.iter().for_each(|stat| println!("{:?}", stat));

    Ok(())
}
