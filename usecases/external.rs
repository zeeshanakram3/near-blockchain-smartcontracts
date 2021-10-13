use super::*;

#[ext_contract(ext_oracle)]
pub trait ExtOracleContract {
    fn request(
        &mut self,
        callback_address: AccountId,
        callback_method: String,
        nonce: Option<U128>,
        data: Base64String,
    );
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_get_nft_token(&mut self, user_id: String, oracle_account: AccountId);
    fn transfer_ft_and_update_nft(&mut self, nonce: String);
    fn distribute_rewards(&mut self, nonce: String);
    fn create_twitter_campaign_callback(&self) -> String;
    fn check_subscription_status_callback(&self) -> String;
    fn create_subscription_pass_callback(&self) -> String;
    fn buy_subscription_pass_callback(&self, max_renewals: U64);
    fn create_subscription_pass_oracle_request(&self, pass_expires_at: u64) -> String;
    fn renew_subscription_pass_callback(&mut self) -> String;
    fn on_get_storage_balance(&mut self, request_data: JSONCampaign, oracle_account: AccountId);
}

#[ext_contract(ext_contract)]
pub trait ExtContract {
    fn nft_mint(
        &mut self,
        metadata: TokenMetadata,
        receiver_id: ValidAccountId,
        creator_id: Option<ValidAccountId>,
        token_type: String,
    ) -> String;
    fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: U128);
    fn ft_transfer_from(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
    );
    fn nft_tokens_for_type(
        &self,
        token_type: String,
        from_index: U64,
        limit: U64,
    ) -> Vec<JsonToken>;
    fn ext_nft_token(&self, token_id: String) -> String;
    fn nft_token(&self, token_id: String) -> JsonToken;
    fn nft_update(&mut self, token_id: String, metadata: TokenMetadata) -> String;
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> AccountStorageBalance;
    fn storage_balance_of(&self, account_id: ValidAccountId) -> AccountStorageBalance;
}
