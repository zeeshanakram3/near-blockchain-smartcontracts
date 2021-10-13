use crate::*;

use near_sdk::json_types::ValidAccountId;
use near_sdk::{ext_contract, Gas, PromiseResult};

const GAS_FOR_RESOLVE_TRANSFER: Gas = 5_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

const NO_DEPOSIT: Balance = 0;

pub trait SocialTokensCore {
    fn st_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: Option<ValidAccountId>,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
    );
    fn st_transfer_call(
        &mut self,
        token_symbol: Symbol,
        receiver_id: ValidAccountId,
        amount: U128,
        msg: String,
        memo: Option<String>,
    ) -> Promise;
    fn get_social_token_supply(&self, token_symbol: Symbol) -> Balance;
    fn get_social_token_owner(&self, token_symbol: Symbol) -> AccountId;
    fn get_balance(&self, token_symbol: Symbol, account_id: AccountId) -> Balance;
}

#[ext_contract(ext_social_token_receiver)]
trait SocialTokensReceiver {
    /// Called by fungible token contract after `ft_transfer_call` was initiated by
    /// `sender_id` of the given `amount` with the transfer message given in `msg` field.
    /// The `amount` of tokens were already transferred to this contract account and ready to be used.
    fn st_on_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait SocialTokensResolverExt {
    fn st_resolve_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

trait SocialTokensResolver {
    fn st_resolve_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

#[near_bindgen]
impl SocialTokensCore for SocialTokens {
    #[payable]
    fn st_transfer(
        &mut self,
        token_symbol: Symbol,
        mut sender_id: Option<ValidAccountId>,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        if sender_id == None {
            sender_id = Some(
                near_sdk::json_types::ValidAccountId::try_from(env::predecessor_account_id())
                    .unwrap(),
            );
        };

        let amount = amount.into();
        self.internal_transfer(
            token_symbol,
            &sender_id.unwrap().into(),
            receiver_id.as_ref(),
            amount,
            memo,
        );
    }

    #[payable]
    fn st_transfer_call(
        &mut self,
        token_symbol: Symbol,
        receiver_id: ValidAccountId,
        amount: U128,
        msg: String,
        memo: Option<String>,
    ) -> Promise {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount = amount.into();
        self.internal_transfer(
            token_symbol.clone(),
            &sender_id,
            receiver_id.as_ref(),
            amount,
            memo,
        );
        // Initiating receiver's call and the callback
        ext_social_token_receiver::st_on_transfer(
            token_symbol.clone(),
            sender_id.clone(),
            amount.into(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::st_resolve_transfer(
            token_symbol,
            sender_id,
            receiver_id.into(),
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
    }

    fn get_social_token_supply(&self, token_symbol: Symbol) -> Balance {
        match self.get_social_token(token_symbol) {
            Some(st) => st.supply,
            None => env::panic(b"Social token with given symbol does not exist."),
        }
    }

    fn get_social_token_owner(&self, token_symbol: Symbol) -> AccountId {
        match self.get_social_token(token_symbol) {
            Some(metadata) => metadata.creator_id,
            None => env::panic(b"Social token with given symbol does not exist."),
        }
    }

    fn get_balance(&self, token_symbol: Symbol, account_id: AccountId) -> Balance {
        self.st_get_balance(token_symbol, &account_id)
            .expect(&(account_id.clone() + " account is not registered"))
    }
}

#[near_bindgen]
impl SocialTokensResolver for SocialTokens {
    fn st_resolve_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        assert_self();
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount, // TODO why
        };

        if unused_amount > 0 {
            let receiver_balance = self
                .st_get_balance(token_symbol.clone(), &receiver_id)
                .unwrap_or(0);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                self.st_set_balance(
                    token_symbol.clone(),
                    &receiver_id,
                    receiver_balance - refund_amount,
                );
                if let Some(sender_balance) = self.st_get_balance(token_symbol.clone(), &sender_id)
                {
                    self.st_set_balance(token_symbol, &sender_id, sender_balance + refund_amount);
                    env::log(
                        format!(
                            "Refund {} from {} to {}",
                            refund_amount, receiver_id, sender_id
                        )
                        .as_bytes(),
                    );
                    return (amount - refund_amount).into();
                } else {
                    // Sender's account was deleted, so we need to burn tokens.
                    let mut social_token = self.get_social_token(token_symbol.clone()).unwrap();
                    social_token.supply -= refund_amount;
                    env::log(b"The account of the sender was deleted");
                    env::log(format!("Burn {}", refund_amount).as_bytes());
                    self.set_social_token(token_symbol, &social_token)
                }
            }
        }
        amount.into() // TODO: i think this should be something else, how many were returned
    }
}
