use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{self, UnorderedSet, UnorderedMap};
use near_sdk::json_types::{Base58PublicKey, Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Promise, PromiseOrValue};


#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

//Implemented below is the Tender Factory Implementation and the process of sending NEAR to commit onto implementing a tender issued by a small business or merchant

mod utils;
use crate::utils::*

// Estimating that it will require at least 30 NEAR tokens to store a single tender, could still change(Issue)
const MIN_ATTACHED_BALANCE: Balance = 30_000_000_000_000_000_000_000_000;

// Feature to include, a helper function to calculate storage cost of a tender created before hand and then price how much it would cost to issue/post a tender



pub mod gas {
    use near_sdk::Gas;

    /// The base amount of gas for a regular execution.
    const BASE: Gas = 25_000_000_000_000;

    /// The amount of Gas the contract will attach to the promise to create the staking pool.
    /// The base for the execution and the base for staking action to verify the staking key.
    pub const STAKING_POOL_NEW: Gas = BASE * 2;

    /// The amount of Gas the contract will attach to the callback to itself.
    /// The base for the execution and the base for whitelist call or cash rollback.
    pub const CALLBACK: Gas = BASE * 2;

    /// The amount of Gas the contract will attach to the promise to the verifying tender contract(borrows the concept of whitelisting staking pool contracts.
    /// The base for the execution.
    pub const VERIFY_TENDER: Gas = BASE;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TenderFactory {
    /// Account ID of the tenders created
    tender_account_ids: UnorderedSet<AccountId>,

    /// Account ID of the Verify Tender Contract.
    /// The verify account implementation mimics the idea of the whiteli    ///st contract with a few alterations
    verify_tender_account_id: AccountId,

}

impl Default for TenderFactory {
    fn default() -> Self {
        env::panic(b"The contract should be initialized before usage")
    }
}


pub struct TenderParameters {
    // Owner account ID of the tender issued
    owner_id: AccountId,
    // Public key initiated to secure the tender
    tender_public_key: Base58PublicKey,
    // Tender proposal statement
    tender_proposal: String,
    // Product/service needed
    product: String,
    // Unit price of product/service needed
    unitproductprice: U128,
    // Quantity of product/service needed
    quantityproduct: u64,
    // Industry/Sector of the Tender originator
    industry: String,
    // Location of Delivery for the Product/Service
    location: String,
}


/// External interface for the callbacks to self
#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn create_tender(
       &mut self,
       tender_account_id: AccountId,
       posting_fee: U128,
       predecessor_account_id: AccountId,
    ) -> Promise;
}


/// External interface for the Verify Tender(whitelist) contract.
pub trait ExVerifyTender {
    fn add_tender(&mut self, tender_account_id: AccountId) -> bool;
}


#[near_bindgen]
impl TenderFactory {
     /// Initializes the tender factory with the given account ID of the    ///Verify tender(whitelist) contract
     #[init]
     pub fn new(verify_tender_account_id: AccountId) -> Self {
     	 assert!(!env::state_exists(), "The contract is already initialized");
	 assert!(
	     env::is_valid_account_id(verify_tender_account_id.as_bytes()), "The verify tender account ID is invalid");
	     Self {
	         verify_tender_account_id,
		 tender_account_ids: UnorderedSet::new(b"s".to_vec()),
	     }
     }


     /// Returns the minimum amount of tokens needed to attach to the fu    ///nction call to create a new tender.
    pub fn get_min_attached_balance(&self) -> U128 {
        MIN_ATTACHED_BALANCE.into()
    }
    

    /// Returns the total number of tenders created from this factory
    pub fn get_number_of_tenders_created(&self) -> U64 {
        self.tender_account_ids.len()
    }


    /// Creates a new tender
    #[payable]
    pub fn create_tender(
        &mut self,
	tender_registration_id: String,
	owner_id: AccountId,
	tender_public_key: Base58PublicKey,
	tender_proposal: String,
        product: String,
        unitproductprice: U128,
        quantityproduct: u64,
        industry: String,
        location: String,
    ) -> Promise {
        assert!(
	    // To change this and add a proper fee for tender creation t	    //aking into account gas costs for storage
	    env::attached_deposit() = MIN_ATTACHED_BALANCE,
	    "Not enough attached deposit to issue the tender"
	);

	assert!(
	    tender_registration_id.find('.').is_none(),
	    "The tender registration ID can't contain `.`"
	);

	let tender_account_id = format!("{}.{}", tender_registration_id, env::current_account_id());
        assert!(
            env::is_valid_account_id(tender_account_id.as_bytes()),
            "The tender account ID is invalid"
        );

	assert!(
	    env::is_valid_account_id(owner_id.as_bytes()),
	    "The owner account ID is invalid"
	);

	assert!(
	    self.tender_account_ids.insert(&tender_account_id),
	    "The tender account ID already exists"
	);


	Promise::new(tender_account_id.clone())
	    .create_account()
	    .transfer(env::attached_deposit())
	    .deploy_contract(include_bytes!("../../tender/res/tender.wasm").to_vec())
	    .function_call(
	        b"new".to_vec(),
		near_sdk::serde_json::to_vec(&TenderParameters {
		    owner_id,
		    tender_public_key,
		    //---to add more tender parameters--
		})
		.unwrap(),
		NO_DEPOSIT,
		gas::TENDER_NEW,
	    )
	    .then(ext_self::on_tender_create(
	        tender_account_id,
		env::attached_deposit().into(),
		env::predecessor_account_id(),
		&env::current_account_id(),
		NO_DEPOSIT,
		gas::CALLBACk,
	    ))

}


/// Callback function after a tender was created
/// Returns the promise to verify the tender contract if the tender crea///tion was successful
/// If not then it refunds the attached deposit and returns `false`.
pub fn on_tender_create(
    &mut self,
        tender_account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
	//---To Add More Parameters--
    ) -> PromiseOrValue<bool> {
        assert_self();

        let tender_created = is_promise_success();

        if tender_created {
            env::log(
                format!(
                    "The tender @{} was successfully created. Securing...",
                    tender_account_id
                )
                .as_bytes(),
            );
            ext_whitelist::add_tender(
                tender_account_id,
                &self.verify_tender_account_id,
                NO_DEPOSIT,
                gas::VERIFY_TENDER,
            )
            .into()
        } else {
            self.tender_account_ids
                .remove(&tender_account_id);
            env::log(
                format!(
                    "The tender @{} creation process has failed. Returning attached deposit of {} to @{}",
                    tender_account_id,
                    attached_deposit.0,
                    predecessor_account_id
                ).as_bytes()
            );
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            PromiseOrValue::Value(false)
        }
    }
}
		    

	
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, MockedBlockchain, PromiseResult};

    mod test_utils;
    use std::convert::TryInto;
    use test_utils::*;

    #[test]
    fn test_create_tender_success() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tenderbox())
	    //Tenderbox account is the account of the holding co.
            .finish();
        testing_env!(context.clone());

        let mut contract = TenderFactory::new(account_verify_tender());

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_tenders_created(), 0);

        context.is_view = false;
        context.predecessor_account_id = account_tokens_owner();
        context.attached_deposit = ntoy(31);
        testing_env!(context.clone());
        contract.create_tender(
            tender_id(),
            account_tender_owner(),
            "KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .try_into()
                .unwrap(),
        );

        context.predecessor_account_id = account_factory();
        context.attached_deposit = ntoy(0);
        testing_env_with_promise_results(context.clone(), PromiseResult::Successful(vec![]));
        contract.on_tender_create(account_pool(), ntoy(31).into(), account_tokens_owner());

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_number_of_tenders_created(), 1);
    }

    #[test]
    #[should_panic(expected = "Not enough attached deposit to complete tender creation")]
    fn test_create_tender_not_enough_deposit() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = TenderFactory::new(account_verify_tender());

        // Checking the pool is still whitelisted
        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_tender_created(), 0);

        context.is_view = false;
        context.predecessor_account_id = account_tokens_owner();
        context.attached_deposit = ntoy(20);
        testing_env!(context.clone());
        contract.create_tender(
            tender_registration_id(),
            account_tender_owner(),
            "KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .try_into()
                .unwrap(),
	);
    }

    #[test]
    fn test_create_tender_rollback() {
        let mut context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tenderbox())
            .finish();
        testing_env!(context.clone());

        let mut contract = TenderFactory::new(account_verify());

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_tender_created(), 0);

        context.is_view = false;
        context.predecessor_account_id = account_tokens_owner();
        context.attached_deposit = ntoy(31);
        testing_env!(context.clone());
        contract.create_tender(
            tender_registration_id(),
            account_tender_owner(),
            "KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .try_into()
                .unwrap(),
            
        );

        context.predecessor_account_id = account_factory();
        context.attached_deposit = ntoy(0);
        context.account_balance += ntoy(31);
        testing_env_with_promise_results(context.clone(), PromiseResult::Failed);
        let res = contract.on_tender_create(
            account_pool(),
            ntoy(31).into(),
            account_tokens_owner(),
        );
        match res {
            PromiseOrValue::Promise(_) => panic!("Unexpected result, should return Value(false)"),
            PromiseOrValue::Value(value) => assert!(!value),
        };

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_number_of_tenders_created(), 0);
    }
}
