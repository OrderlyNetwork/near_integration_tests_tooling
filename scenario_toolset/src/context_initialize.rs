use crate::{
    contract_initializer::ContractInitializer,
    print_log,
    utils::{token_info::TokenInfo, TestAccount},
};
use anyhow::Ok;
use futures::{
    future::try_join_all, stream::FuturesUnordered, try_join, FutureExt, StreamExt, TryFutureExt,
};
use near_sdk::{json_types::U128, Balance};
use near_units::parse_near;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use test_token::TokenContractTest;
use tokio::task::JoinHandle;
use workspaces::{
    network::Sandbox,
    types::{KeyType, SecretKey},
    Account, AccountId, Contract, Worker,
};

const JOIN_MAX: usize = 500;
const JOIN_CHUNK: usize = 100;

pub async fn initialize_context<T, U, const N: usize, const M: usize>(
    token_infos: &[TokenInfo; N],
    test_accounts: &[TestAccount; M],
    contract_initializer: &(impl ContractInitializer<T, U> + Sync + Send),
) -> anyhow::Result<(Worker<Sandbox>, T, U, [TokenContractTest; N], [Account; M])> {
    let worker = workspaces::sandbox().await?;
    let contract_wasm = contract_initializer.get_wasm();

    let (contract, contract_accounts, token_contracts, accounts) = try_join!(
        worker.create_tla_and_deploy(
            contract_initializer.get_id(),
            SecretKey::from_random(KeyType::ED25519),
            &contract_wasm
        ),
        try_join_all(contract_initializer.get_role_accounts().into_iter().map(
            |(
                role,
                TestAccount {
                    account_id,
                    mint_amount,
                },
            )| {
                worker
                    .create_tla(account_id.clone(), SecretKey::from_random(KeyType::ED25519))
                    .map(|result| {
                        result.map(|account| {
                            (
                                role,
                                account,
                                TestAccount {
                                    account_id,
                                    mint_amount,
                                },
                            )
                        })
                    })
            }
        )),
        try_join_all(token_infos.iter().map(|token_info| {
            worker.create_tla_and_deploy(
                token_info.account_id.clone(),
                SecretKey::from_random(KeyType::ED25519),
                &token_info.wasm_file,
            )
        })),
        try_join_all(test_accounts.iter().take(JOIN_MAX).map(
            |TestAccount { account_id, .. }| {
                worker.create_tla(account_id.clone(), SecretKey::from_random(KeyType::ED25519))
            }
        ))
    )?;

    let contract = contract.into_result()?;
    let contract_accounts = contract_accounts
        .into_iter()
        .map(|(role, account, mint_amount)| account.into_result().map(|el| (role, el, mint_amount)))
        .collect::<Result<Vec<(String, Account, TestAccount)>, _>>()?;

    print_log!("Created contract {}", contract.as_account().id().blue());

    let token_contracts_and_infos = token_contracts
        .into_iter()
        .zip(token_infos.iter())
        .map(|(token_contract, token_info)| {
            token_contract
                .into_result()
                .map(|el| (el, token_info.clone()))
        })
        .collect::<Result<Vec<(Contract, TokenInfo)>, _>>()?
        .into_iter()
        .map(|(token_contract, token_info)| {
            print_log!("Created token {}", token_contract.as_account().id().blue());
            (
                token_info,
                TokenContractTest {
                    contract: token_contract,
                    measure_storage_usage: false,
                },
            )
        })
        .collect::<Vec<(_, _)>>();

    initialize_tokens(token_contracts_and_infos.iter()).await?;

    let mut accounts = accounts
        .into_iter()
        .map(|test_account| test_account.into_result())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|test_account| {
            print_log!("Created account {}", test_account.id().bright_green());
            test_account
        })
        .collect::<Vec<Account>>();

    accounts.extend(create_rest_of_accounts(&worker, test_accounts).await?);

    make_storage_deposits_and_mint_tokens(
        &token_contracts_and_infos,
        contract.as_account().id(),
        &accounts,
        test_accounts,
        contract_accounts
            .iter()
            .map(|(_, account, test_account)| (account, test_account)),
    )
    .await?;

    let account_details = contract.as_account().view_account().await?;
    print_log!(
        "{:.3} {} contract storage usage after init",
        (account_details.storage_usage as f64 / 100_000.)
            .bright_magenta()
            .bold(),
        "NEAR".bright_magenta().bold()
    );

    let (template, holder) = contract_initializer
        .initialize_contract_template(
            contract,
            HashMap::from_iter(
                contract_accounts
                    .iter()
                    .map(|(role, account, _)| (role.clone(), account.clone())),
            ),
        )
        .await?;

    Ok((
        worker,
        template,
        holder,
        token_contracts_and_infos
            .into_iter()
            .map(|(_, token_contract)| token_contract)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        accounts.try_into().unwrap(),
    ))
}

async fn create_rest_of_accounts(
    worker: &Worker<Sandbox>,
    test_accounts: &[TestAccount],
) -> anyhow::Result<Vec<Account>> {
    let mut accounts = Vec::new();

    let mut account_tasks: Vec<JoinHandle<anyhow::Result<Account>>> = vec![];

    for TestAccount { account_id, .. } in test_accounts.iter().skip(JOIN_MAX) {
        let worker = worker.clone();
        let account_id = account_id.clone();
        account_tasks.push(tokio::spawn(async move {
            let account = worker
                .create_tla(account_id, SecretKey::from_random(KeyType::ED25519))
                .await?
                .into_result()?;
            Ok(account)
        }));

        if account_tasks.len() >= 200 {
            for task in account_tasks.drain(..JOIN_CHUNK) {
                let account = task.await??;
                accounts.push(account);
            }
        }
    }

    for task in account_tasks {
        let account = task.await??;
        accounts.push(account);
    }

    Ok(accounts)
}

async fn initialize_tokens(
    token_contract_and_infos: impl Iterator<Item = &(TokenInfo, TokenContractTest)>,
) -> anyhow::Result<Vec<integration_tests_toolset::tx_result::TxResult<()>>> {
    try_join_all(token_contract_and_infos.map(
        |(
            TokenInfo {
                name,
                ticker,
                decimals,
                ..
            },
            test_token_contract,
        )| {
            test_token_contract.new(
                name.to_string(),
                ticker.to_string(),
                *decimals,
                test_token_contract.contract.as_account(),
            )
        },
    ))
    .await
    .map_err(|err| err.into())
}

async fn make_storage_deposit(
    token_contract: &TokenContractTest,
    storage_deposit: Balance,
    account_id: &AccountId,
    caller: &Account,
) -> anyhow::Result<()> {
    token_contract
        .storage_deposit(Some(account_id.clone()), None, caller, storage_deposit)
        .await?;

    print_log!(
        "Storage deposit account {} for {}",
        account_id.green(),
        token_contract.contract.id().blue()
    );
    Ok(())
}

async fn mint_tokens(
    token_contract: &TokenContractTest,
    account_id: &AccountId,
    caller: &Account,
    amount: u128,
) -> anyhow::Result<()> {
    token_contract
        .mint(
            account_id.clone(),
            U128(amount),
            caller,
            parse_near!("1 yN"),
        )
        .await?;

    print_log!(
        "Minted {} {} for {}",
        amount.to_string().cyan(),
        token_contract.contract.as_account().id().blue(),
        account_id.green()
    );

    Ok(())
}

async fn make_storage_deposits_and_mint_tokens(
    token_contracts_and_infos: &Vec<(TokenInfo, TokenContractTest)>,
    contract_id: &AccountId,
    accounts: &[Account],
    test_accounts: &[TestAccount],
    contract_accounts: impl Iterator<Item = (&Account, &TestAccount)> + Clone,
) -> anyhow::Result<()> {
    let futures = FuturesUnordered::new();
    for (token_info, token_contract) in token_contracts_and_infos.iter() {
        for (account, TestAccount { mint_amount, .. }) in accounts.iter().zip(test_accounts.iter())
        {
            if let Some(amount) = mint_amount.get(&token_info.account_id.to_string()) {
                futures.push(
                    make_storage_deposit(
                        token_contract,
                        token_info.storage_deposit_amount,
                        account.id(),
                        account,
                    )
                    .and_then(|_| mint_tokens(token_contract, account.id(), account, *amount))
                    .boxed(),
                );
            }
        }
        for (account, TestAccount { mint_amount, .. }) in contract_accounts.clone() {
            if let Some(amount) = mint_amount.get(&token_info.account_id.to_string()) {
                futures.push(
                    make_storage_deposit(
                        token_contract,
                        token_info.storage_deposit_amount,
                        account.id(),
                        account,
                    )
                    .and_then(|_| mint_tokens(token_contract, account.id(), account, *amount))
                    .boxed(),
                );
            }
        }
        futures.push(
            make_storage_deposit(
                token_contract,
                token_info.storage_deposit_amount,
                contract_id,
                token_contract.contract.as_account(),
            )
            .boxed(),
        );
    }

    futures
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
