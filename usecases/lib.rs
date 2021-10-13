use std::collections::HashMap;
use std::str;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::utils::promise_result_as_success;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Promise, PromiseOrValue};
use std::convert::TryFrom;

use external::*;
use helpers::*;

mod callbacks;
mod external;
pub mod helpers;
#[cfg(test)]
mod test;
mod trait_iplms;
mod usecase_subscription;
mod usecase_twitter_campaign;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const SINGLE_CALL_GAS: u64 = 30_000_000_000_000;
const BASE_CALL_GAS: u64 = 25_000_000_000_000;
const TWITTER_CAMPAIGN_NFT_TYPE: &str = "reward";
const CONTENT_NFT_TYPE: &str = "content";
const SUBSCRIPTION_PASS_NFT_TYPE: &str = "subscription";
const NFT_CONTRACT_ACCOUNT: &str = "nft.momentize.testnet";
const FT_CONTRACT_ACCOUNT: &str = "ft.momentize.testnet";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenId(String);
pub type Base64String = String;
pub type CampaignId = u128;
pub type OracleCallback = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UsecasesContract {
    // TODO only one reward against
    owner_id: AccountId,
    trusted_oracles: Vec<AccountId>,
    request_nonce: u128,
    nonce_owner: UnorderedMap<String, ValidAccountId>,
    received: UnorderedMap<String, OracleCallback>,
}

impl Default for UsecasesContract {
    fn default() -> Self {
        panic!("Usecases client should be initialized before usage")
    }
}

#[near_bindgen]
impl UsecasesContract {
    #[allow(dead_code)]
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            trusted_oracles: Vec::new(),
            request_nonce: 0_u128,
            nonce_owner: UnorderedMap::new(b"nonce_owner".to_vec()),
            received: UnorderedMap::new(b"received".to_vec()),
        }
    }

    #[allow(dead_code)]
    pub fn add_oracle(&mut self, oracle_account: ValidAccountId) {
        assert!(
            env::signer_account_id() == self.owner_id,
            "Only account owner can add oracle account"
        );
        if !self
            .trusted_oracles
            .contains(&oracle_account.clone().into())
        {
            self.trusted_oracles.push(oracle_account.into())
        }
    }

    #[allow(dead_code)]
    pub fn remove_oracle(&mut self, oracle_account: AccountId) {
        assert!(
            env::signer_account_id() == self.owner_id,
            "Only account owner can remove oracle account"
        );

        let index = self
            .trusted_oracles
            .iter()
            .position(|x| *x == oracle_account)
            .unwrap();
        self.trusted_oracles.remove(index);
    }
}
