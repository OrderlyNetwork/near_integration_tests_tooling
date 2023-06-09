use integration_tests_bindgen_macro::integration_tests_bindgen;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{assert_one_yocto, env, json_types::U128, near_bindgen, AccountId, PromiseOrValue};

#[integration_tests_bindgen]
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct TestContract {
    state: u64,
}

/// Test contract for checking test bindgen macro and scenario toolset.
/// Generate test contract for integration tests.
/// Contract contains different types of methods and parameters.
#[integration_tests_bindgen]
#[near_bindgen]
impl TestContract {
    #[init]
    #[payable]
    #[private]
    #[handle_result]
    pub fn new(initial_state: u64) -> Result<Self, &'static str> {
        assert_one_yocto();
        if initial_state > 10 {
            return Err("initial state should be less than 10");
        }

        Ok(Self {
            state: initial_state,
        })
    }

    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        #[derive(BorshDeserialize)]
        struct OldContract {
            _state: u64,
        }

        let _old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self { state: 0 }
    }

    pub fn view_no_param_ret_u64(&self) -> u64 {
        self.state
    }

    pub fn view_param_account_id_ret_account_id(&self, account: AccountId) -> AccountId {
        account
    }

    pub fn view_param_tuple_with_account_id(&self, tuple: (AccountId, u64)) -> AccountId {
        tuple.0
    }

    pub fn view_param_arr_tuples_with_account_id(
        &self,
        arr_tuple: [(AccountId, u64); 1],
    ) -> AccountId {
        arr_tuple.first().unwrap().0.clone()
    }

    pub fn view_param_vec_tuple_of_vec_tuples_with_account_id(
        &self,
        vec_tuple: Vec<(Vec<(AccountId, u64)>, u64)>,
    ) -> AccountId {
        vec_tuple.first().unwrap().0.first().unwrap().0.clone()
    }

    pub fn view_param_vec_tuple_with_account_id(
        &self,
        vec_tuple: Vec<(AccountId, u64)>,
    ) -> AccountId {
        vec_tuple.first().unwrap().0.clone()
    }

    pub fn view_param_vec_account_id_ret_vec_account_id(
        &self,
        accounts: Vec<AccountId>,
    ) -> Vec<AccountId> {
        accounts
    }

    #[handle_result]
    pub fn view_no_param_ret_error_handle_res(&self) -> Result<(), &'static str> {
        Err("View function raised error!")
    }

    pub fn view_account_id(&self, account: AccountId) -> AccountId {
        account
    }

    pub fn view_ref_account_id(&self, account: &AccountId) -> AccountId {
        account.clone()
    }

    pub fn view_option_account_id(&self, account: Option<AccountId>) -> Option<AccountId> {
        account
    }

    pub fn call_no_param_ret_u64(&mut self) -> u64 {
        self.state += 1;
        self.state
    }

    #[handle_result]
    pub fn call_param_u64_ret_u64_handle_res(
        &mut self,
        increase_for: u64,
    ) -> Result<u64, &'static str> {
        self.state = self.state.checked_add(increase_for).ok_or("error")?;
        Ok(self.state)
    }

    /// Unconditionally panics.
    #[handle_result]
    pub fn call_no_param_ret_error_handle_res(&mut self) -> Result<(), &'static str> {
        Err("Call function rised error!")
    }

    #[payable]
    pub fn call_no_param_no_ret_payable(&mut self) {
        assert_one_yocto();

        self.state += 1
    }
}

#[integration_tests_bindgen]
#[near_bindgen]
#[allow(unused_variables)]
impl FungibleTokenReceiver for TestContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        PromiseOrValue::Value(0.into())
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, ONE_YOCTO};

    use super::*;

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    #[test]
    fn test_() -> Result<(), &'static str> {
        let context = VMContextBuilder::new().attached_deposit(ONE_YOCTO).build();
        testing_env!(context);
        let mut contract = TestContract::new(1)?;

        assert_eq!(contract.view_no_param_ret_u64(), 1);

        contract.call_no_param_ret_u64();

        assert_eq!(contract.view_no_param_ret_u64(), 2);

        contract.call_no_param_no_ret_payable();

        assert_eq!(contract.view_no_param_ret_u64(), 3);
        Ok(())
    }
}
