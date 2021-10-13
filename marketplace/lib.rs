mod external;
mod ft_callbacks;
mod internal;
mod nft_callbacks;
mod sale;
mod sale_views;

near_sdk::setup_alloc!();

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, CryptoHash, Gas,
    PanicOnDefault, Promise, PromiseOrValue,
};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::external::*;
use crate::internal::*;
use crate::sale::*;
use near_sdk::env::STORAGE_PRICE_PER_BYTE;

// TODO check seller supports storage_deposit at ft_token_id they want to post sale in

const NO_DEPOSIT: Balance = 0;
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
static DELIMETER: &str = "||";
static MARKETPLACE_ACCOUNT_ID: &str = "marketplace.momentize.testnet";
static FT_ACCOUNT_ID: &str = "ft.momentize.testnet";

pub type TokenId = String;
pub type TokenType = Option<String>;
pub type FTOrSTIdAndStSymbol = String;
pub type ContractAndTokenId = String;
// TODO: Capital U128
pub type Payout = HashMap<AccountId, U128>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Marketplace {
    pub owner_id: AccountId,
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub by_nft_token_type: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub ft_token_ids: UnorderedSet<AccountId>, // in case of st
    pub storage_deposits: LookupMap<AccountId, Balance>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
}

#[near_bindgen]
impl Marketplace {
    #[init]
    pub fn new(owner_id: ValidAccountId, ft_token_ids: Option<Vec<ValidAccountId>>) -> Self {
        let mut this = Self {
            owner_id: owner_id.into(),
            sales: UnorderedMap::new(StorageKey::Sales.try_to_vec().unwrap()),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId.try_to_vec().unwrap()),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId.try_to_vec().unwrap()),
            by_nft_token_type: LookupMap::new(StorageKey::ByNFTTokenType.try_to_vec().unwrap()),
            ft_token_ids: UnorderedSet::new(StorageKey::FTTokenIds.try_to_vec().unwrap()),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits.try_to_vec().unwrap()),
        };
        // support NEAR by default
        this.ft_token_ids.insert(&"near".to_string());
        if let Some(ft_token_ids) = ft_token_ids {
            for ft_token_id in ft_token_ids {
                this.ft_token_ids.insert(ft_token_id.as_ref());
            }
        }

        this
    }

    /// only owner
    pub fn add_ft_or_st_token_ids(
        &mut self,
        mut ft_or_st_token_id: ValidAccountId,
        st_symbol: Option<String>,
    ) -> bool {
        self.assert_owner();
        if let Some(st_symbol) = st_symbol {
            ft_or_st_token_id = ValidAccountId::try_from(format!(
                "{}{}{}",
                ft_or_st_token_id, DELIMETER, st_symbol
            ))
            .unwrap();
        }
        self.ft_token_ids.insert(ft_or_st_token_id.as_ref())
    }

    /// TODO remove token (should check if sales can complete even if owner stops supporting token type)

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) {
        let storage_account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(env::predecessor_account_id);
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
        let sales = self.by_owner_id.get(&owner_id);
        let len = if sales.is_some() {
            sales.unwrap().len()
        } else {
            0
        };
        amount -= u128::from(len) * STORAGE_PER_SALE;
        if amount > 0 {
            Promise::new(owner_id).transfer(amount);
        }
    }

    /// views

    pub fn supported_ft_token_ids(&self) -> Vec<AccountId> {
        self.ft_token_ids.to_vec()
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_paid(&self, account_id: ValidAccountId) -> U128 {
        U128(self.storage_deposits.get(account_id.as_ref()).unwrap_or(0))
    }
}
