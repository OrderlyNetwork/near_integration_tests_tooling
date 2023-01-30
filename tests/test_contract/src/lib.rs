use integration_tests_bindgen_macro::integration_tests_bindgen;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{assert_one_yocto, env, near_bindgen};

#[integration_tests_bindgen]
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TestContract {
    state: u64,
}

impl Default for TestContract {
    fn default() -> Self {
        Self { state: 0 }
    }
}

#[integration_tests_bindgen]
#[near_bindgen]
impl TestContract {
    #[init]
    pub fn new() -> Self {
        Self { state: 0 }
    }

    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        #[derive(BorshDeserialize)]
        struct OldContract {}

        let _old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self { state: 0 }
    }

    pub fn test_change_state(&mut self) {
        self.state += 1
    }

    pub fn test_get_state(&self) -> u64 {
        self.state
    }

    #[payable]
    pub fn test_payable_change_state(&mut self) {
        assert_one_yocto();

        self.state += 1
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, ONE_YOCTO};

    use super::*;

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    #[test]
    fn test_() {
        let mut contract = TestContract::new();

        assert_eq!(contract.test_get_state(), 0);

        contract.test_change_state();

        assert_eq!(contract.test_get_state(), 1);

        let context = VMContextBuilder::new().attached_deposit(ONE_YOCTO).build();
        testing_env!(context);

        contract.test_payable_change_state();

        assert_eq!(contract.test_get_state(), 2);
    }
}
