use super::*;

#[near_bindgen]
impl UsecasesContract {
    #[payable]
    pub fn create_subscription_pass(
        &mut self,
        content_reference: String,
        pass_price: U128,
        ft_account_id: AccountId,
    ) -> PromiseOrValue<String> {
        // create subscription NFTs
        ext_contract::nft_mint(
            TokenMetadata {
                title: Some("Subscription content NFT".to_string()),
                description: Some("Subscription content NFT".to_string()),
                reference: Some(content_reference),
                ft_account_id: Some(ft_account_id),
                ft_amount: Some(pass_price),
                max_updates: None,
                extra: None,
                update_no: None,
                issued_at: None,
                starts_at: None,
                updated_at: None,
                expires_at: None,
            },
            ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),
            Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
            CONTENT_NFT_TYPE.to_string(),
            &NFT_CONTRACT_ACCOUNT.to_string(),
            env::attached_deposit(),
            SINGLE_CALL_GAS,
        )
        .into()
    }

    #[payable]
    pub fn buy_subscription_pass(&mut self, content_id: String, max_renewals: U64) {
        ext_contract::nft_token(content_id, &NFT_CONTRACT_ACCOUNT, 0, SINGLE_CALL_GAS).then(
            ext_self::buy_subscription_pass_callback(
                max_renewals,
                &env::current_account_id(),
                env::attached_deposit(),
                7 * SINGLE_CALL_GAS,
            ),
        );
    }

    pub fn check_subscription_status(&self, subscription_pass_id: String) {
        // check NFT metadata for expiration
        ext_contract::nft_token(
            subscription_pass_id.to_string(),
            &NFT_CONTRACT_ACCOUNT,
            0,
            SINGLE_CALL_GAS,
        )
        .then(ext_self::check_subscription_status_callback(
            &env::current_account_id(),
            0,
            5 * SINGLE_CALL_GAS,
        ));
    }

    #[payable]
    pub fn renew_subscription_pass(&mut self, subscription_pass_id: String) {
        ext_contract::nft_token(
            subscription_pass_id.to_string(),
            &NFT_CONTRACT_ACCOUNT,
            0,
            SINGLE_CALL_GAS,
        )
        .then(ext_self::renew_subscription_pass_callback(
            &env::current_account_id(),
            0,
            5 * SINGLE_CALL_GAS,
        ));
    }
}
