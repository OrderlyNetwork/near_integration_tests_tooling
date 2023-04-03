# Integration Tests Bindgen Macro and Toolset
## Quick Start

- Install [Rustup](https://rustup.rs/)
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- Install wasm-opt 0.110.0
```shell
cargo install --version 0.110.0 wasm-opt
```
- Build the project
```shell
./build.sh
```
- Run the tests
```shell
cargo test -- --nocapture
```
## Description of the problem , that this toolset solves
When you are developing smart contracts, you need to test them. There are two types of tests: unit tests and integration tests. Unit tests are testing your contract functions in isolation, without any other contracts, accounts, tokens, etc. Integration tests are testing your contract functions in combination with other contracts, accounts, tokens, etc. 

Integration tests are more complex, because you need to deploy contracts, create accounts, mint tokens, etc. Usually it is solved by creating test context structure, that will be used to initialize context for your tests.

This framework intnded to avoid writing test context structure in every contract, minimize boilerplate code for integration tests, by generating contract test structure and functions, initialize context for your tests, provide tools for creating test scenarios, catch and print error inside transactions, provide statistic consumers/processors.

There are three parts of this framework:
- integration_tests_bindgen_macro - procedural macro, that generates contract test structure and functions for integration tests of you contract, that can be used separately from other parts of this framework.
- initialize_context function from scenario_toolset - function, that aimed to substitute test context structure, simplify and speed up initialization of context for your tests.
- scenario_toolset - set of tools for creating batch operations scenarios.

## Generating Contract Test structure and functions for integration tests of you contract
Calling contract functions from integration tests is not trivial, because you need to write some bilerplate code for that. To simplify smart contract function calling, immediately obtain result value from it, obtain transaction logs and statistics, and print them, you can use generated contract test structure and functions.

**Note! generating Test structure and functions for your contract is not adding any runtime code and overhead to your contract, it is adding test functions and code, that run only in integration tests.**

To use generated contract test structure and functions you need to add next lines to your contract Cargo.toml file:
```toml
[dependencies]
integration_tests_bindgen_macro = {path = "local-path-to/near_integration_tests_tooling/integration_tests_bindgen_macro"}
integration_tests_toolset = {path = "local-path-to/near_integration_tests_tooling/integration_tests_toolset"}

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
anyhow = "1"
async-trait = "0.1"
integration_tests_bindgen_macro = {path = "local-path-to/near_integration_tests_tooling/integration_tests_bindgen_macro"}
lazy_static = "1"
scenario_toolset = {path = "local-path-to/near_integration_tests_tooling/scenario_toolset"}
workspaces = "0.7"
```
Then add #[integration_tests_bindgen] attribute to your contract mod:
```rust
use integration_tests_bindgen_macro::integration_tests_bindgen;

#[integration_tests_bindgen]
#[near_bindgen]
pub struct TestContract {
```
Also you need to add #[integration_tests_bindgen] attribute to impl blocks of your contract with functions you want to generate test functions for:
```rust
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::contract::TestContractTest;

#[integration_tests_bindgen]
#[near_bindgen]
impl TestContract {
```
In the integration tests you have to initialize generated Test structure (for `TestContract` it will be `TestContractTest`) with deployed workspace::Contract structure and use it's functions to call contract functions:
```rust
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

    contract_template
        .new(10, &contract_template.contract.as_account(), 1u128)
        .await?
        .populate_statistic(&mut statistic_consumers)
        .print_statistic()?;
```
Example of test contract with generated test structure and functions can be found in the `/tests/test_contract` folder.
Example of usage of generated test structure and functions can be found in the `/tests/tests/only_test_gen.rs` file.

## Using initialize_context
Usually you need to initialize context for your contract tests, for example, deploy contract, deploy test fungible tokens contract, create accounts, mint tokens, etc. To simplify this process you can use initialize_context function from scenario_toolset. 

It will create contract, test accounts and tokens, mint tokens to test accounts according to provided TestAccount.mint_amount fields and return initialized ContractTest structure, ContractHolder with contract role accounts, initialized test token contracts, initialized test accounts, with minted tokens. It is creating test accounts, deploy contract and tokens in parralel manner as fast as possible.

To use initialize_context you need to implement ContractInitializer structure for your contract, that will be used to initialize contract and role accounts. 
Then you can use initialize_context function to initialize context for your contract tests:

```rust
#[tokio::test]
async fn block_operations_example() -> anyhow::Result<()> {
    let (worker, contract_template, contract_holder, [_eth, _usdc], [maker_account]) = initialize_context(
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
```

Example of ContractInitializer implementation for test contract can be found in the `tests/tests/contract_initializer.rs` file.
Example of usage of initialize_context function can be found in the `tests/tests/contract_initializer_test.rs` file.

## Using batch operations
Batch operations intended to create test scenarios, where commands can be executed in sequence or in parallel in any combinations. 
Example of batch operations usage:
```rust
use scenario_toolset::{
    batch::{make_op, make_unit_op, Batch},
    context_initialize::initialize_context,};

#[tokio::test]
async fn test_batch() -> anyhow::Result<()> {
    let (_, contract_template, _, _, _) = initialize_context(&[], &[], &Initializer {}).await?;
    Batch::new()
        .add_chain_op(make_op(contract_template.view_no_param_ret_u64()))
        .run()
        .await?
        .process_statistic([
            Box::new(GasUsage::default()),
            Box::new(StorageUsage::default()),
        ])
        .print_statistic()?;

    Ok(())
}
```

Examples of batch operations usage can be found in the `tests/tests/batch_operations.rs` file.

## Exploring The Code

1. The procedural macro, #[integration_tests_bindgen], that generates contract test structure and functions lives in the `/integration_tests_bindgen_macro` folder.
2. Integration test toolset, including:
    - structures, used in generated contract test functions,
    - statistics consumers/processors (there are three pre-defined statistic processors: GasUsage, StorageUsage, CallCounter)
    lives in the `/integration_tests_toolset` folder.
3. Toolset for test scenarios:
    - context initialization, including contract deployment and initialization, deployment of test fungible tokens, account creation, token minting,
    - batch operation execution (with statistic processing and printing),
    lives in the `/scenario_toolset` folder.
4. Test fungible tokens contract used in tests and for usage in target contract tests lives in the `/test_fungible_tokens` folder.
5. Test contract used in tests with examples of different contract operations, both view and change, lives in the `/tests/test_contract` folder.
6. Different integration tests for Single contract operations, Batch operations, and statistics processing and printing lives in the `/tests/tests` folder:
    - only_test_gen.rs - example of standalone usage of test contract operatios, generated by #[integration_tests_bindgen] macro,
    - contract_initializer.rs - example of ContractInitializer, required for initialize_context
    - contract_initializer_test.rs - example of initialize_context function usage,
    - batch_operations.rs - Different variants of Batch operations with statistic processing and printing,
    - operation_examples.rs - example of custom operations that can be used in batch operations,
    - test_ft_token.rs - example of usage of test fungible tokens contract,

### TODOs

Compile a TODO list via:

```sh
sudo npm i -g leasot
leasot -x -T test --reporter markdown 'integration_tests_bindgen_macro/src/**/*.rs' 'integration_tests_toolset/src/**/*.rs' > TODO.md 
```