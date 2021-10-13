use crate::*;

/// external contract calls

#[ext_contract(nft_sale)]
trait ExtContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: U64,
        memo: Option<String>,
        balance: U128,
    );
    fn nft_revoke(&mut self, token_id: TokenId, account_id: ValidAccountId);
    fn ft_transfer(
        &mut self,
        sender_id: Option<ValidAccountId>,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
    fn st_transfer(
        &mut self,
        token_symbol: String,
        sender_id: Option<ValidAccountId>,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
}
