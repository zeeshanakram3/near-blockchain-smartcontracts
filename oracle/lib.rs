use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::u128::MAX;
use std::{str, u128};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{TreeMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PromiseResult;
use near_sdk::{env, near_bindgen, AccountId};
use serde_json::json;

mod helpers;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const EXPIRY_TIME: u64 = 5 * 60 * 1_000_000_000;

// max gas: 300_000_000_000_000

const MINIMUM_CONSUMER_GAS_LIMIT: u64 = 1_000_000_000;
const SINGLE_CALL_GAS: u64 = 100_000_000_000_000; // 5 x 10^13

pub type Base64String = String;

#[derive(Default, BorshDeserialize, BorshSerialize, Debug, Clone, Serialize, Deserialize)]
pub struct OracleRequest {
    callback_address: AccountId,
    callback_method: String,
    data: Base64String,
    expiration: u64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct RequestsJSON {
    nonce: Option<U128>,
    request: OracleRequest,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Oracle {
    pub owner: AccountId,
    pub nonces: TreeMap<AccountId, Option<U128>>,
    pub requests: TreeMap<AccountId, TreeMap<Option<u128>, OracleRequest>>,
    pub authorized_nodes: UnorderedSet<AccountId>,
}

impl Default for Oracle {
    fn default() -> Self {
        panic!("Oracle should be initialized before usage")
    }
}

#[near_bindgen]
impl Oracle {
    /// Initializes the contract with the given total supply owned by the given `owner_id` and `withdrawable_tokens`
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: owner_id,
            nonces: TreeMap::new(b"nonces".to_vec()),
            requests: TreeMap::new(b"requests".to_vec()),
            authorized_nodes: UnorderedSet::new(b"authorized_nodes".to_vec()),
        }
    }

    pub fn request(
        &mut self,
        callback_address: AccountId,
        callback_method: String,
        nonce: Option<U128>,
        data: Base64String,
    ) {
        self._check_callback_address(&callback_address);

        let entry_option = self.requests.get(&env::predecessor_account_id());
        if entry_option.is_some() && nonce.is_some() {
            // Ensure there isn't already the same nonce
            let nonce_entry = entry_option.unwrap();
            if nonce_entry.contains_key(&Some(nonce.unwrap().0)) {
                env::panic(b"Existing account and nonce in requests");
            }
        }

        let last_nonce_option: Option<Option<U128>> = self.get_nonce(env::predecessor_account_id());
        if last_nonce_option.is_some() && nonce.is_some() {
            if let Some(last_nonce_option) = last_nonce_option {
                let last_nonce_u128: u128 = last_nonce_option.unwrap_or(U128(MAX)).into();
                assert!(
                    last_nonce_u128 < nonce.unwrap().0,
                    "Invalid, already used nonce: {} {}",
                    nonce.unwrap().0,
                    last_nonce_u128
                );
            }
        }
        self.store_request(
            env::predecessor_account_id(),
            callback_address,
            callback_method,
            nonce,
            data,
        )
    }

    /// Accounts/contracts should call `request`, which in turn calls this contract via a promise
    #[allow(unused_variables)]
    pub fn store_request(
        &mut self,
        sender: AccountId,
        callback_address: AccountId,
        callback_method: String,
        nonce: Option<U128>,
        data: Base64String,
    ) {
        let nonce_u128: Option<u128> = if let Some(nonce) = nonce {
            Some(nonce.into())
        } else {
            None
        };
        let expiration: u64 = env::block_timestamp() + EXPIRY_TIME;

        // store request
        let oracle_request = OracleRequest {
            callback_address,
            callback_method,
            // data: near_sdk::serde_json::from_str(&data).expect("Not valid request object"),
            data,
            expiration,
        };

        let nonce_request_entry = self.requests.get(&sender);
        let mut nonce_request = if nonce_request_entry.is_none() {
            TreeMap::new(sender.clone().into_bytes())
        } else {
            nonce_request_entry.unwrap()
        };
        nonce_request.insert(&nonce_u128, &oracle_request);
        self.requests.insert(&sender.clone(), &nonce_request);
        self.nonces.insert(&sender.clone(), &nonce.clone());
        env::log(
            format!(
                "Inserted request with\nKey: {:?}\nValue: {:?}",
                nonce.clone(),
                oracle_request.clone()
            )
            .as_bytes(),
        );
    }

    #[payable]
    pub fn fulfill_request(&mut self, account: AccountId, nonce: U128, data: Base64String) {
        self._only_authorized_node();

        // TODO: this is probably going to be too low at first, adjust
        assert!(
            env::prepaid_gas() - env::used_gas() > MINIMUM_CONSUMER_GAS_LIMIT,
            "Must provide consumer enough gas"
        );

        // Get the request
        let account_requests = self.requests.get(&account);
        if account_requests.is_none() {
            env::panic(b"Did not find the account to fulfill.");
        }
        let nonce_u128: u128 = nonce.into();
        let request_option = account_requests.unwrap().get(&Some(nonce_u128));
        if request_option.is_none() {
            env::panic(b"Did not find the request (nonce) to fulfill.");
        }
        let request = request_option.unwrap();
        env::log(format!("{}", env::prepaid_gas()).as_bytes());
        let promise_perform_callback = env::promise_create(
            request.callback_address,
            request.callback_method.as_bytes(),
            json!({
                "nonce": nonce.clone(),
                "answer": data
            })
            .to_string()
            .as_bytes(),
            env::attached_deposit(),
            2 * SINGLE_CALL_GAS,
        );

        let promise_post_callback = env::promise_then(
            promise_perform_callback,
            env::current_account_id(),
            b"fulfillment_post_callback",
            json!({
                "account": account,
                "nonce": nonce
            })
            .to_string()
            .as_bytes(),
            0,
            SINGLE_CALL_GAS / 2,
        );

        env::promise_return(promise_post_callback);
    }

    pub fn fulfillment_post_callback(&mut self, account: AccountId, nonce: U128) {
        self._only_owner_predecessor();
        // TODO: fix this "if" workaround until I can figure out how to write tests with promises
        if cfg!(target_arch = "wasm32") {
            assert_eq!(env::promise_results_count(), 1);
            // ensure successful promise, meaning tokens are transferred
            match env::promise_result(0) {
                PromiseResult::Successful(_) => {}
                PromiseResult::Failed => env::panic(
                    b"(fulfillment_post_callback) The promise failed. See receipt failures.",
                ),
                PromiseResult::NotReady => env::panic(b"The promise was not ready."),
            };
        }
        // Remove request from state
        let mut account_requests = self.requests.get(&account).unwrap();
        let nonce_u128: u128 = nonce.into();
        account_requests.remove(&Some(nonce_u128));
        // Must overwrite the new TreeMap with the account key
        self.requests.insert(&account, &account_requests);
        env::log(b"Request has completed successfully and been removed.");
    }

    pub fn is_authorized(&self, node: AccountId) -> bool {
        self.authorized_nodes.contains(&node)
    }

    pub fn add_authorization(&mut self, node: AccountId) {
        self._only_owner();
        assert!(
            env::is_valid_account_id(node.as_bytes()),
            "Account ID is invalid"
        );
        self.authorized_nodes.insert(&node);
    }

    pub fn remove_authorization(&mut self, node: AccountId) {
        self._only_owner();

        self.authorized_nodes.remove(&node);
    }

    pub fn get_requests(&self, account: AccountId, max_requests: U64) -> Vec<RequestsJSON> {
        let max_requests_u64: u64 = max_requests.into();
        if !self.requests.contains_key(&account) {
            env::panic(format!("Account {} has no requests.", account).as_bytes());
        }
        let mut counter: u64 = 0;
        let mut result: Vec<RequestsJSON> = Vec::with_capacity(max_requests_u64 as usize);
        let account_requests_map = self.requests.get(&account).unwrap();

        for req in account_requests_map.iter() {
            self._request_iterate(&max_requests_u64, req, &mut result, &mut counter);
        }

        result
    }

    /// Helper function while iterating through account requests
    fn _request_iterate(
        &self,
        max_requests: &u64,
        req: (Option<u128>, OracleRequest),
        result: &mut Vec<RequestsJSON>,
        counter: &mut u64,
    ) {
        if *counter == *max_requests || *counter > self.requests.len() {
            return;
        }
        let nonce = req.0;
        let oracle_request = req.1;

        let request_nonce: Option<U128> = if let Some(_nonce) = nonce {
            Some(_nonce.into())
        } else {
            None
        };
        result.push(RequestsJSON {
            nonce: request_nonce,
            request: oracle_request,
        });

        *counter += 1;
    }

    pub fn get_all_requests(
        &self,
        max_num_accounts: U64,
        max_requests: U64,
    ) -> HashMap<AccountId, Vec<RequestsJSON>> {
        let max_requests_u64: u64 = max_requests.into();
        let max_num_accounts_u64: u64 = max_num_accounts.into();
        let mut account_counter: u64 = 0;
        let mut result: HashMap<AccountId, Vec<RequestsJSON>> = HashMap::new();

        for account_requests in self.requests.iter() {
            if account_counter == max_num_accounts_u64 || account_counter > self.requests.len() {
                break;
            }
            let mut requests: Vec<RequestsJSON> = Vec::new();
            let mut request_counter: u64 = 0;
            for nonce_request in account_requests.1.iter() {
                if request_counter == max_requests_u64 || request_counter > account_requests.1.len()
                {
                    break;
                }

                let request_nonce: Option<U128> = if let Some(_nonce) = nonce_request.0 {
                    Some(_nonce.into())
                } else {
                    None
                };
                let req = RequestsJSON {
                    nonce: request_nonce,
                    request: nonce_request.1,
                };
                requests.push(req);
                request_counter += 1;
            }
            result.insert(account_requests.0.clone(), requests);
            account_counter += 1;
        }
        result
    }

    pub fn get_campaign_requests(
        &self,
        max_num_accounts: U64,
        max_requests: U64,
    ) -> HashMap<AccountId, Vec<RequestsJSON>> {
        let max_requests_u64: u64 = max_requests.into();
        let max_num_accounts_u64: u64 = max_num_accounts.into();
        let mut account_counter: u64 = 0;
        let mut result: HashMap<AccountId, Vec<RequestsJSON>> = HashMap::new();

        for account_requests in self.requests.iter() {
            if account_counter == max_num_accounts_u64 || account_counter > self.requests.len() {
                break;
            }
            let mut requests: Vec<RequestsJSON> = Vec::new();
            let mut request_counter: u64 = 0;
            for nonce_request in account_requests.1.iter() {
                if request_counter == max_requests_u64 || request_counter > account_requests.1.len()
                {
                    break;
                }

                let request_nonce: Option<U128> = if let Some(_nonce) = nonce_request.0 {
                    Some(_nonce.into())
                } else {
                    None
                };
                if nonce_request.1.callback_method == "campaign_verification_callback".to_string() {
                    let req = RequestsJSON {
                        nonce: request_nonce,
                        request: nonce_request.1,
                    };
                    requests.push(req);
                    request_counter += 1;
                }
            }
            result.insert(account_requests.0.clone(), requests);
            account_counter += 1;
        }
        result
    }
    pub fn get_subscription_renewal_requests(
        &self,
        max_num_accounts: U64,
        max_requests: U64,
    ) -> HashMap<AccountId, Vec<RequestsJSON>> {
        let max_requests_u64: u64 = max_requests.into();
        let max_num_accounts_u64: u64 = max_num_accounts.into();
        let mut account_counter: u64 = 0;
        let mut result: HashMap<AccountId, Vec<RequestsJSON>> = HashMap::new();

        for account_requests in self.requests.iter() {
            if account_counter == max_num_accounts_u64 || account_counter > self.requests.len() {
                break;
            }
            let mut requests: Vec<RequestsJSON> = Vec::new();
            let mut request_counter: u64 = 0;
            for nonce_request in account_requests.1.iter() {
                if request_counter == max_requests_u64 || request_counter > account_requests.1.len()
                {
                    break;
                }

                let request_nonce: Option<U128> = if let Some(_nonce) = nonce_request.0 {
                    Some(_nonce.into())
                } else {
                    None
                };
                if nonce_request.1.callback_method == "renew_subscription_pass".to_string() {
                    let req = RequestsJSON {
                        nonce: request_nonce,
                        request: nonce_request.1,
                    };
                    requests.push(req);
                    request_counter += 1;
                }
            }
            result.insert(account_requests.0.clone(), requests);
            account_counter += 1;
        }
        result
    }
}
