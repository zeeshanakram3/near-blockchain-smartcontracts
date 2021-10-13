/**
* Fungible Token NEP-141 Token contract
*
* The aim of the contract is to provide a basic implementation of the improved function token standard.
*
* lib.rs is the main entry point.
* fungible_token_core.rs implements NEP-146 standard
* storage_manager.rs implements NEP-145 standard for allocating storage per account
* fungible_token_metadata.rs implements NEP-148 standard for providing token-specific metadata.
* internal.rs contains internal methods for fungible token.
*/
mod fungible_token_core;
mod fungible_token_metadata;
mod internal;
mod storage_manager;

near_sdk::setup_alloc!();

pub use crate::fungible_token_core::*;
pub use crate::fungible_token_metadata::*;
pub use crate::storage_manager::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::utils::{assert_one_yocto, assert_self};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, StorageUsage};
use std::convert::TryFrom;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub owner_id: AccountId,

    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, Balance>,

    /// (OwnerId, EscrowId) -> Account balance.
    pub allowances: LookupMap<(AccountId, AccountId), Balance>,

    /// Total supply of the all token.
    pub total_supply: Balance,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    pub ft_metadata: FungibleTokenMetadata,
}

impl Default for FungibleToken {
    fn default() -> Self {
        env::panic(b"FungibleToken Contract is not initialized");
    }
}

#[near_bindgen]
impl FungibleToken {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: U128,
        version: Option<String>,
        name: String,
        symbol: String,
        reference: Option<String>,
        reference_hash: Option<[u8; 32]>,
        decimals: u8,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Self {
            owner_id: owner_id.clone().into(),
            accounts: LookupMap::new(b"a".to_vec()),
            allowances: LookupMap::new(b"a".to_vec()),
            total_supply: total_supply.into(),
            account_storage_usage: 0,
            ft_metadata: FungibleTokenMetadata {
                version,
                name,
                symbol,
                reference,
                reference_hash,
                decimals,
            },
        };
        // Determine cost of insertion into LookupMap
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = unsafe { String::from_utf8_unchecked(vec![b'a'; 64]) };
        this.accounts.insert(&tmp_account_id, &0u128);
        this.account_storage_usage = env::storage_usage() - initial_storage_usage;
        this.accounts.remove(&tmp_account_id);
        // Make owner have total supply
        let total_supply_u128: u128 = total_supply.into();
        this.accounts.insert(&owner_id.as_ref(), &total_supply_u128);
        this
    }

    /// Custom Methods
    pub fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: U128) {
        let allowance = allowance.into();
        let owner_id = env::signer_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }

        if allowance > 0 {
            self.allowances
                .insert(&(owner_id, escrow_account_id), &allowance);
        } else {
            self.allowances.remove(&(owner_id, escrow_account_id));
        }
    }

    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> Balance {
        match self.allowances.get(&(owner_id, escrow_account_id)) {
            Some(allowance) => allowance,
            None => 0,
        }
    }
    /// only owner can mint
    pub fn mint(&mut self, amount: U128) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "must be owner_id"
        );
        self.total_supply += u128::from(amount);
        let mut balance = self
            .accounts
            .get(&self.owner_id)
            .expect("owner should have balance");
        balance += u128::from(amount);
        self.accounts.insert(&self.owner_id, &balance);
    }
}
