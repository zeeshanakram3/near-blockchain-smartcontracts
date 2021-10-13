use crate::*;

/// external contract calls

#[ext_contract(ft_contract)]
trait ExtContract {
    fn ft_transfer(
        &mut self,
        sender_id: Option<ValidAccountId>,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> AccountStorageBalance;
}
