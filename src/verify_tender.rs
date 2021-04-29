use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupSet;
use near_sdk::{env, near_bindgen, AccountId};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;


// Foundation referred in the contracts is the Tenderbox foundation/comp// any that is in charge of the whole Tendering platform

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct VerifyTenderContract {
    /// The account ID of the Tenderbox. It allows to automatically approve and secure newly created Tenders.
    /// We can also verify newly created Tender Factory instances.
    pub foundation_account_id: AccountId,

    /// The verified account IDs of approved Tender contracts.
    pub verified: LookupSet<AccountId>,

    /// The verified list of Tender factories. Any account from this lis   ///t can verify tenders.
    pub factory_verified: LookupSet<AccountId>,
}

impl Default for VerifyTenderContract {
    fn default() -> Self {
        env::panic(b"The contract should be initialized before usage")
    }
}

#[near_bindgen]
impl VerifyTenderContract {
    /// Initializes the contract with the given Tender account ID.
    #[init]
    pub fn new(foundation_account_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(
            env::is_valid_account_id(foundation_account_id.as_bytes()),
            "The Tenderbox account ID is invalid"
        );
        Self {
            foundation_account_id,
            verified: LookupSet::new(b"w".to_vec()),
            factory_verified: LookupSet::new(b"f".to_vec()),
        }
    }



    /// Returns `true` if the given tender account ID is verified.
    pub fn is_verified(&self, staking_pool_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(tender_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.verified.contains(&tender_account_id)
    }

    /// Returns `true` if the given factory contract account ID is whitelisted.
    pub fn is_factory_verified(&self, factory_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.factory_verified.contains(&factory_account_id)
    }

    /************************/
    /* Tender Factory + Tenderbox Foundation */
    /************************/

    /// Adds the given tender account ID to the verified list.
    /// Returns `true` if the tender was not verified before, `false` otherwise.
    /// This method can be called either by the Tenderbox foundation/company or by a verified factory.
    pub fn add_tender(&mut self, tender_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(tender_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        // Can only be called by a verified factory or by the foundation.
        if !self
            .factory_verified
            .contains(&env::predecessor_account_id())
        {

	     self.assert_called_by_foundation();
        }
        self.verified.insert(&tender_account_id)
    }

    /**************/
    /* Tenderbox Foundation */
    /**************/

    /// Removes the given tender account ID from the list of verified tenders(verified).
    /// Returns `true` if the tender was present in the verified tenders' list before, `false` otherwise.
    /// This method can only be called by Tenderbox Foundation(Guardian company.
    pub fn remove_tender(&mut self, staking_pool_account_id: AccountId) -> bool {
        self.assert_called_by_foundation();
        assert!(
            env::is_valid_account_id(tender_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.verified.remove(&tender_account_id)
    }

    /// Adds the given tender factory contract account ID to the list of verified Tender Factories.
    /// Returns `true` if the factory was not in the verified list before, `false` otherwise.
    /// This method can only be called by the Tenderbox foundation.
    pub fn add_factory(&mut self, factory_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.assert_called_by_foundation();
        self.factory_whitelist.insert(&factory_account_id)
    }

    /// Removes the given tender factory account ID from the list of verified factories.
    /// Returns `true` if the factory was present in the list of verified factories before, `false` otherwise.
    /// This method can only be called by the Tenderbox foundation.
    pub fn remove_factory(&mut self, factory_account_id: AccountId) -> bool {
        self.assert_called_by_foundation();
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.factory_verified.remove(&factory_account_id)
    }

    /************/
    /* Internal */
    /************/

    /// Internal method to verify the predecessor was the Tenderbox Foundation account ID.
    fn assert_called_by_foundation(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.foundation_account_id,
            "Can only be called by the Tenderbox Foundation"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, MockedBlockchain};

    mod test_utils;
    use test_utils::*;

    #[test]
    fn test_verified() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_verified())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = VerifiedTenderContract::new(account_near());

        // Check initial list of verified tenders
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_verified(account_tender()));

        // Adding to verified list by foundation
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.add_tender(account_tender()));

        // Checking it's verified now
        context.is_view = true;
        testing_env!(context.clone());
        assert!(contract.is_verified(account_tender()));

        // Adding again. Should return false
        context.is_view = false;
        testing_env!(context.clone());
        assert!(!contract.add_tender(account_tender()));

        // Checking the pool is still verified
        context.is_view = true;
        testing_env!(context.clone());
        assert!(contract.is_verified(account_pool()));

        // Removing from the list of verified tenders(called verified).
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.remove_tender(account_tender()));

        // Checking the pool is not verified anymore
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_verified(account_pool()));

        // Removing again from the whitelist, should return false.
        context.is_view = false;
        testing_env!(context.clone());
        assert!(!contract.remove_tender(account_tender()));

        // Checking the pool is still not verified
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_verified(account_pool()));

        // Adding again after it was removed. Should return true
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.add_tender(account_tender()));

        // Checking the pool is now verified again
        context.is_view = true;
        testing_env!(context.clone());
        assert!(contract.is_verified(account_pool()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by Tenderbox Foundation")]
    fn test_factory_verified_fail() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_verified())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = VerifyTenderContract::new(account_tenderbox());

        // Trying ot add to the verified list by NOT verified factory.
        context.is_view = false;
        context.predecessor_account_id = account_factory();
        testing_env!(context.clone());
        assert!(contract.add_tender(account_tender()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by Tenderbox Foundation")]
    fn test_trying_to_verify_factory() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_verified())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = VerifyTenderContract::new(account_tenderbox());

        // Trying to verify the factory not initiated by the Tenderbox Foundation.
        context.is_view = false;
        context.predecessor_account_id = account_tenderfactory();
        testing_env!(context.clone());
        assert!(contract.add_factory(account_tenderfactory()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by Tenderbox Foundation")]
    fn test_trying_to_remove_by_factory() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_verified())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = VerifyTenderContract::new(account_tenderbox());

        // Adding factory
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.add_factory(account_factory()));

        // Trying to remove the tender by the factory.
        context.predecessor_account_id = account_factory();
        testing_env!(context.clone());
        assert!(contract.remove_tender(account_tender()));
    }

    #[test]
    fn test_verified_factory() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_verified())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = TenderboxContract::new(account_tenderbox());

        // Check the factory is not verified
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_factory_verified(account_factory()));

        // Verified factory
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.add_factory(account_factory()));

        // Check the factory is verified now(whitelisted)
        context.is_view = true;
        testing_env!(context.clone());
        assert!(contract.is_factory_verified(account_factory()));
        // Check the tender is not verified
        assert!(!contract.is_verified(account_tender()));

        // Adding to list of verified tenders by foundation
        context.is_view = false;
        context.predecessor_account_id = account_factory();
        testing_env!(context.clone());
        assert!(contract.add_tender(account_tender()));

        // Checking it's verified now
        context.is_view = true;
        testing_env!(context.clone());
        assert!(contract.is_verified(account_pool()));

        // Removing the tender from the list of verified tenders by the Tenderbox foundation.
        context.is_view = false;
        context.predecessor_account_id = account_tenderbox();
        testing_env!(context.clone());
        assert!(contract.remove_tender(account_tender()));

        // Checking the tender is not verified anymore
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_verified(account_tender()));

        // Removing the factory
        context.is_view = false;
        testing_env!(context.clone());
        assert!(contract.remove_factory(account_factory()));

        // Check the factory is not verified anymore
        context.is_view = true;
        testing_env!(context.clone());
        assert!(!contract.is_factory_verified(account_factory()));
    }
}