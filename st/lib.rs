mod bonding_curve;
mod external;
mod internal;
mod social_token;
mod social_token_core;
mod social_tokens_metadata;
mod storage_manager;

near_sdk::setup_alloc!();

pub use crate::bonding_curve::*;
use crate::external::*;
pub use crate::social_token::*;
pub use crate::social_tokens_metadata::*;
pub use crate::storage_manager::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::utils::{assert_one_yocto, assert_self};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, Promise, StorageUsage};
use std::convert::TryFrom;
use std::ops::Add;

type Symbol = String;
type SymbolHash = Vec<u8>;
type SymbolPlusAccountIdHash = Vec<u8>;

const GAS_FOR_FT_TRANSFER: Gas = 5_000_000_000_000;
const DEPOSIT_FOR_FT_ACCOUNT_REGISTRATION: u128 = 1_250_000_000_000_000_000_000;
static ST_ACCOUNT_ID: &str = "st.momentize.testnet";
static FT_ACCOUNT_ID: &str = "ft.momentize.testnet";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SocialTokens {
    pub owner_id: AccountId,

    pub social_tokens: UnorderedMap<SymbolHash, SocialToken>,

    pub social_tokens_by_creator_id: UnorderedMap<AccountId, SymbolHash>,

    pub social_tokens_per_account: LookupMap<AccountId, (SymbolHash, Balance)>,

    pub st_to_accounts: UnorderedMap<SymbolPlusAccountIdHash, Balance>,

    pub account_storage_usage: StorageUsage,

    pub st_metadata: SocialTokensMetadata,
}

impl Default for SocialTokens {
    fn default() -> Self {
        panic!("social token contract must be initialized before usage")
    }
}

#[near_bindgen]
impl SocialTokens {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        version: Option<String>,
        name: String,
        reference: Option<String>,
        reference_hash: Option<[u8; 32]>,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id: owner_id.clone().into(),
            social_tokens: UnorderedMap::new(b"social-tokens".to_vec()),
            st_to_accounts: UnorderedMap::new(b"st_to_accounts".to_vec()),
            social_tokens_by_creator_id: UnorderedMap::new(b"st_by_acc".to_vec()),
            social_tokens_per_account: LookupMap::new(b"st_per_account".to_vec()),
            account_storage_usage: 0,
            st_metadata: SocialTokensMetadata {
                version,
                name,
                reference,
                reference_hash,
            },
        }
    }

    // TODO Create separate ft_account for every token
    #[payable]
    pub fn create_social_token(&mut self, token_symbol: String) {
        assert_one_yocto();
        if let Some(_) = self.get_social_token(token_symbol.clone()) {
            env::panic(b"Social token with given symbol already exists")
        }
        let mut social_token =
            SocialToken::new(env::predecessor_account_id(), token_symbol.clone());
        ft_contract::storage_deposit(
            Some(social_token.collateral_account_id.clone()),
            &FT_ACCOUNT_ID.to_string(),
            DEPOSIT_FOR_FT_ACCOUNT_REGISTRATION,
            GAS_FOR_FT_TRANSFER,
        );
        // at least 1 ft required for st creation
        ft_contract::ft_transfer(
            Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
            social_token.collateral_account_id.clone().into(),
            U128(1),
            None,
            &FT_ACCOUNT_ID.to_string(),
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .as_return();

        // update collateral and supply locally
        social_token.collateral_amount += 1;
        social_token.supply += 100;

        self.set_social_token(token_symbol.clone(), &social_token);
        let symbol_hash = env::sha256(token_symbol.as_bytes());
        self.social_tokens_by_creator_id
            .insert(&env::predecessor_account_id(), &symbol_hash);

        // Determine cost of insertion into LookupMap
        let initial_storage_usage = env::storage_usage();
        let owned_string = token_symbol.to_owned();
        let tmp_symbol_plus_account_id =
            env::sha256(owned_string.add(&env::predecessor_account_id()).as_bytes());
        self.st_to_accounts
            .insert(&tmp_symbol_plus_account_id, &0u128);
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        self.st_to_accounts.remove(&tmp_symbol_plus_account_id);
    }

    #[payable]
    pub fn st_mint(&mut self, token_symbol: String, collateral_amount: U128) {
        ft_contract::ft_transfer(
            Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
            self.st_collateral_account_id(token_symbol.clone()).into(),
            collateral_amount,
            None,
            &FT_ACCOUNT_ID.to_string(),
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .as_return();
        self.continuous_mint(collateral_amount.into(), token_symbol)
    }

    #[payable]
    pub fn st_burn(&mut self, token_symbol: String, burn_amount: U128) {
        ft_contract::ft_transfer(
            Some(self.st_collateral_account_id(token_symbol.clone())),
            env::predecessor_account_id(),
            burn_amount,
            None,
            &FT_ACCOUNT_ID.to_string(),
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .as_return();
        self.continuous_burn(burn_amount.into(), token_symbol)
    }
}
