use super::*;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::Serialize;

/// Price per 1 byte of storage from mainnet config after `0.18` release and protocol version `42`.
/// It's 10 times lower than the genesis price.
pub const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStorageBalance {
    total: U128,
    available: U128,
}

pub trait StorageManager {
    fn storage_deposit(
        &mut self,
        token_symbol: Symbol,
        account_id: Option<ValidAccountId>,
    ) -> AccountStorageBalance;

    fn storage_withdraw(&mut self, token_symbol: Symbol, amount: U128) -> AccountStorageBalance;

    fn storage_minimum_balance(&self) -> U128;

    fn storage_balance_of(
        &self,
        token_symbol: Symbol,
        account_id: ValidAccountId,
    ) -> AccountStorageBalance;
}

#[near_bindgen]
impl StorageManager for SocialTokens {
    #[payable]
    fn storage_deposit(
        &mut self,
        token_symbol: Symbol,
        account_id: Option<ValidAccountId>,
    ) -> AccountStorageBalance {
        let amount = env::attached_deposit();
        assert_eq!(
            amount,
            self.storage_minimum_balance().0,
            "Requires attached deposit of the exact storage minimum balance"
        );
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let owned_string = token_symbol.to_owned();
        let symbol_account_id_hash = env::sha256(owned_string.add(&account_id).as_bytes());

        if self
            .st_to_accounts
            .insert(&symbol_account_id_hash, &0)
            .is_some()
        {
            env::panic(b"The account is already registered");
        }
        AccountStorageBalance {
            total: amount.into(),
            available: amount.into(),
        }
    }

    #[payable]
    fn storage_withdraw(&mut self, token_symbol: Symbol, amount: U128) -> AccountStorageBalance {
        assert_one_yocto();
        let amount: Balance = amount.into();
        assert_eq!(
            amount,
            self.storage_minimum_balance().0,
            "The withdrawal amount should be the exact storage minimum balance"
        );
        let account_id = env::predecessor_account_id();
        let owned_string = token_symbol.to_owned();
        let symbol_account_id_hash = env::sha256(owned_string.add(&account_id).as_bytes());
        if let Some(balance) = self.st_to_accounts.remove(&symbol_account_id_hash) {
            if balance > 0 {
                env::panic(b"The account has positive token balance");
            } else {
                Promise::new(account_id).transfer(amount + 1);
                AccountStorageBalance {
                    total: 0.into(),
                    available: 0.into(),
                }
            }
        } else {
            env::panic(b"The account is not registered");
        }
    }

    fn storage_minimum_balance(&self) -> U128 {
        (Balance::from(self.account_storage_usage) * STORAGE_PRICE_PER_BYTE).into()
    }

    fn storage_balance_of(
        &self,
        token_symbol: Symbol,
        account_id: ValidAccountId,
    ) -> AccountStorageBalance {
        let owned_string = token_symbol.to_owned();
        let symbol_account_id_hash = env::sha256(owned_string.add(&account_id.as_ref()).as_bytes());
        if let Some(balance) = self.st_to_accounts.get(symbol_account_id_hash.as_ref()) {
            AccountStorageBalance {
                total: self.storage_minimum_balance(),
                available: if balance > 0 {
                    0.into()
                } else {
                    self.storage_minimum_balance()
                },
            }
        } else {
            AccountStorageBalance {
                total: 0.into(),
                available: 0.into(),
            }
        }
    }
}
