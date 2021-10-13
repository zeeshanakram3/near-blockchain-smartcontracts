use std::u128;

use super::*;

const NANOSECS_IN_MONTH: u64 = 2_592_000_000_000_000;
const NANOSECS_IN_HOUR: u64 = 2976190;

#[near_bindgen]
impl UsecasesContract {
    #[allow(dead_code)]
    pub fn create_twitter_campaign_callback(&self) -> String {
        let campaign_promise_res =
            promise_result_as_success().expect("nft_mint(method) callback: promise failed");
        let campaign_id = near_sdk::serde_json::from_slice::<String>(&campaign_promise_res)
            .expect("Not valid NFT campaign_id");
        env::log(format!("campaign_id: {}", campaign_id).as_bytes());
        campaign_id
    }

    #[allow(dead_code)]
    pub fn create_subscription_pass_callback(&self) -> String {
        let subscription_pass_promise_res =
            promise_result_as_success().expect("nft_mint(method) callback: promise failed");
        let subscription_id =
            near_sdk::serde_json::from_slice::<String>(&subscription_pass_promise_res)
                .expect("Not valid NFT suscription_id");
        env::log(format!("suscription_id: {}", subscription_id).as_bytes());
        subscription_id
    }

    #[allow(dead_code)]
    pub fn check_subscription_status_callback(&self) -> bool {
        let subscription_pass_promise_res =
            promise_result_as_success().expect("nft_token(method) callback: promise failed");
        let supbcription_pass =
            near_sdk::serde_json::from_slice::<JsonToken>(&subscription_pass_promise_res)
                .expect("Not valid NFT suscription pass");

        let is_pass_valid = supbcription_pass
            .metadata
            .expires_at
            .expect("subscription pass expiry is not set")
            .parse::<u64>()
            .unwrap_or(0)
            > env::block_timestamp();

        let pass_status = if is_pass_valid {
            "valid".to_string()
        } else {
            "expired".to_string()
        };
        env::log(format!("suscription_pass status: {}", pass_status).as_bytes());
        is_pass_valid
    }

    #[allow(dead_code)]
    pub fn renew_subscription_pass_callback(&mut self) {
        let subscription_pass_promise_res =
            promise_result_as_success().expect("nft_token(method) callback: promise failed");
        let subscription_pass =
            near_sdk::serde_json::from_slice::<JsonToken>(&subscription_pass_promise_res)
                .expect("Not valid NFT suscription pass");

        let is_pass_valid = subscription_pass
            .metadata
            .expires_at
            .clone()
            .expect("subscription pass expiry is not set")
            .parse::<u64>()
            .unwrap_or(0)
            > env::block_timestamp();

        if !is_pass_valid {
            let mut updated_metadata = subscription_pass.metadata.clone();
            updated_metadata.update_no =
                Some(U64(u64::from(updated_metadata.update_no.unwrap()) + 1));
            updated_metadata.expires_at = Some(
                (updated_metadata
                    .expires_at
                    .expect("subscription pass expiry is not set")
                    .parse::<u64>()
                    .expect("failed to parse suscription pass expiry time")
                    + NANOSECS_IN_MONTH)
                    .to_string(),
            );

            ext_contract::ft_transfer_from(
                subscription_pass.owner_id,
                subscription_pass.creator_id,
                subscription_pass.metadata.ft_amount.unwrap(),
                None,
                &FT_CONTRACT_ACCOUNT,
                env::attached_deposit(),
                BASE_CALL_GAS,
            )
            .then(ext_contract::nft_update(
                subscription_pass.token_id,
                updated_metadata,
                &NFT_CONTRACT_ACCOUNT,
                0,
                BASE_CALL_GAS,
            ));
        }
    }

    #[allow(dead_code)]
    pub fn buy_subscription_pass_callback(&self, max_renewals: U64) {
        let subscription_pass_promise_res =
            promise_result_as_success().expect("nft_token(method) callback: promise failed");
        let subscription_pass =
            near_sdk::serde_json::from_slice::<JsonToken>(&subscription_pass_promise_res)
                .expect("Not valid NFT suscription pass");

        assert!(
            subscription_pass.token_type.unwrap() == CONTENT_NFT_TYPE.to_string(),
            "only subscribe to Nft of type `content`"
        );

        let pass_expires_at = env::block_timestamp() + NANOSECS_IN_MONTH;
        // mint subscription NFT and transfer it to buyer
        ext_contract::ft_transfer_from(
            ValidAccountId::try_from(env::signer_account_id()).unwrap(),
            subscription_pass.creator_id.clone(),
            subscription_pass.metadata.ft_amount.unwrap(),
            None,
            &FT_CONTRACT_ACCOUNT,
            1,
            BASE_CALL_GAS,
        )
        .then(ext_contract::nft_mint(
            TokenMetadata {
                title: Some("Subscription pass NFT".to_string()),
                description: Some("Subscription pass NFT".to_string()),
                reference: Some(format!(
                    "{}||{}",
                    NFT_CONTRACT_ACCOUNT.to_string(),
                    subscription_pass.token_id
                )),
                ft_account_id: None,
                ft_amount: None,
                max_updates: Some(max_renewals),
                extra: None,
                update_no: Some(U64(0)),
                issued_at: None,
                starts_at: None,
                updated_at: None,
                expires_at: Some(pass_expires_at.to_string()),
            },
            ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),
            Some(subscription_pass.creator_id),
            SUBSCRIPTION_PASS_NFT_TYPE.to_string(),
            &NFT_CONTRACT_ACCOUNT.to_string(),
            env::attached_deposit(),
            SINGLE_CALL_GAS,
        ))
        .then(ext_self::create_subscription_pass_oracle_request(
            pass_expires_at,
            &env::current_account_id(),
            0,
            2 * SINGLE_CALL_GAS,
        ))
        .then(ext_contract::set_allowance(
            env::current_account_id(),
            U128(max_renewals.0 as u128 * subscription_pass.metadata.ft_amount.unwrap().0),
            &subscription_pass.metadata.ft_account_id.unwrap(),
            0,
            SINGLE_CALL_GAS,
        ));
    }

    #[allow(dead_code)]
    pub fn create_subscription_pass_oracle_request(&self, pass_expires_at: u64) {
        let subscription_pass_promise_res =
            promise_result_as_success().expect("nft_mint(method) callback: promise failed");
        let subscription_pass_id =
            near_sdk::serde_json::from_slice::<String>(&subscription_pass_promise_res)
                .expect("Not valid NFT suscription pass");

        let request_data = SubscriptionPassRequest {
            fulfill_at: (pass_expires_at + 6 * NANOSECS_IN_HOUR).to_string(),
            subscription_pass_id: subscription_pass_id,
        };
        ext_oracle::request(
            env::current_account_id(),
            "renew_subscription_pass".to_string(),
            None,
            near_sdk::serde_json::to_string(&request_data).expect("invalid request object"),
            &self.trusted_oracles[0],
            0,
            SINGLE_CALL_GAS,
        );
    }
}
