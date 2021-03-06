use crate::*;

impl FungibleToken {
    pub(crate) fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self
            .accounts
            .get(&account_id)
            .expect(&(account_id.clone() + " account is not registered"));
        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(&account_id, &new_balance);
        } else {
            env::panic(b"Balance overflow");
        }
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self
            .accounts
            .get(&account_id)
            .expect(&(account_id.clone() + " account is not registered"));
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.accounts.insert(&account_id, &new_balance);
        } else {
            env::panic(b"The account doesn't have enough balance");
        }
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        env::log(format!("Transfer {} from {} to {}", amount, sender_id, receiver_id).as_bytes());
        if let Some(memo) = memo {
            env::log(format!("Memo: {}", memo).as_bytes());
        }
    }
}
