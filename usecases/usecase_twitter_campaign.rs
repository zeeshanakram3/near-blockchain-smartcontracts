use super::*;

#[near_bindgen]
impl UsecasesContract {
    #[allow(dead_code)]
    #[payable]
    pub fn create_twitter_campaign(
        &mut self,
        ft_account_id: AccountId,
        ft_amount: U128,
        max_claimants: U64,
        tweet_id: String,
        operations: Vec<String>,
    ) {
        // create campaign NFTs with twitter/FB post link. with max count.
        ext_contract::set_allowance(
            env::current_account_id(),
            ft_amount,
            &ft_account_id,
            0,
            SINGLE_CALL_GAS,
        )
        .then(ext_contract::nft_mint(
            TokenMetadata {
                title: Some("Twitter campaign NFT".to_string()),
                description: Some("Twitter campaign NFT".to_string()),
                reference: Some(tweet_id),
                ft_account_id: Some(ft_account_id),
                ft_amount: Some(ft_amount),
                max_updates: Some(max_claimants),
                extra: Some(operations),
                update_no: Some(U64(0)),
                issued_at: None,
                starts_at: None,
                updated_at: None,
                expires_at: None,
            },
            ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),
            None,
            TWITTER_CAMPAIGN_NFT_TYPE.to_string(),
            &NFT_CONTRACT_ACCOUNT.to_string(),
            env::attached_deposit(),
            SINGLE_CALL_GAS,
        ))
        .then(ext_self::create_twitter_campaign_callback(
            &env::current_account_id(),
            0,
            SINGLE_CALL_GAS,
        ));
    }

    #[allow(dead_code)]
    #[payable]
    pub fn verify_campaign(
        &mut self,
        user_id: String,
        oracle_account: AccountId,
        campaign_id: CampaignId,
    ) {
        if !self.trusted_oracles.contains(&oracle_account) {
            env::panic(b"oracle account is not trusted");
        }
        ext_contract::nft_token(
            campaign_id.to_string(),
            &NFT_CONTRACT_ACCOUNT,
            0,
            SINGLE_CALL_GAS,
        )
        .then(ext_self::on_get_nft_token(
            user_id,
            oracle_account,
            &env::current_account_id(),
            env::attached_deposit(),
            7 * SINGLE_CALL_GAS,
        ));
    }

    #[allow(dead_code)]
    #[payable]
    pub fn on_get_nft_token(&mut self, user_id: String, oracle_account: AccountId) {
        let campaign_promise_res =
            promise_result_as_success().expect("nft_token(method) callback: promise failed");
        let campaign = near_sdk::serde_json::from_slice::<JsonToken>(&campaign_promise_res)
            .expect("Not valid NFT campaign");

        let update_no: u64 = campaign.metadata.update_no.unwrap().into();
        let max_updates: u64 = campaign.metadata.max_updates.unwrap().into();
        assert!(update_no < max_updates, "No more rewards");

        let request_data = JSONCampaign {
            user_id: user_id,
            campaign_id: campaign.token_id,
            tweet_id: campaign.metadata.reference.unwrap(),
            operations: campaign.metadata.extra.unwrap(),
        };

        ext_contract::storage_balance_of(
            ValidAccountId::try_from(env::signer_account_id()).unwrap(),
            &FT_CONTRACT_ACCOUNT.to_string(),
            0,
            SINGLE_CALL_GAS,
        )
        .then(ext_self::on_get_storage_balance(
            request_data,
            oracle_account,
            &env::current_account_id(),
            env::attached_deposit(),
            5 * SINGLE_CALL_GAS,
        ));
    }

    #[payable]
    pub fn on_get_storage_balance(
        &mut self,
        request_data: JSONCampaign,
        oracle_account: AccountId,
    ) -> U128 {
        let storage_balance_promise_res = promise_result_as_success()
            .expect("storage_balance_of(method) callback: promise failed");
        let storage_balance =
            near_sdk::serde_json::from_slice::<AccountStorageBalance>(&storage_balance_promise_res)
                .expect("Storage balance not found.");

        let storage_deposit_proimse = if 0_u128 >= storage_balance.total.into() {
            ext_contract::storage_deposit(
                Some(ValidAccountId::try_from(env::signer_account_id()).unwrap()),
                &FT_CONTRACT_ACCOUNT.to_string(),
                env::attached_deposit(),
                SINGLE_CALL_GAS,
            )
        } else {
            Promise::new(env::signer_account_id()).transfer(env::attached_deposit())
        };

        storage_deposit_proimse.then(ext_oracle::request(
            env::current_account_id(),
            "campaign_verification_callback".to_string(),
            Some(self.request_nonce.into()),
            near_sdk::serde_json::to_string(&request_data).expect("invalid request object"),
            &oracle_account,
            0,
            SINGLE_CALL_GAS,
        ));

        self.nonce_owner.insert(
            &self.request_nonce.to_string(),
            &ValidAccountId::try_from(env::signer_account_id()).unwrap(),
        );
        self.request_nonce += 1;
        U128(self.request_nonce)
    }

    #[allow(dead_code)] // This function gets called from the oracle
    #[payable]
    pub fn campaign_verification_callback(&mut self, nonce: String, answer: Base64String) {
        if !self
            .trusted_oracles
            .contains(&env::predecessor_account_id())
        {
            env::panic(b"oracle account is not trusted");
        }
        env::log(
            format!(
                "Usecases contract received campaign verification result: {:?}",
                answer
            )
            .as_bytes(),
        );

        let result: CampaignVerificationResult =
            near_sdk::serde_json::from_str(&answer).expect("Not valid verification object");
        self.received.insert(&nonce, &answer);
        if result.completed {
            ext_contract::nft_token(
                result.campaign_id.to_string(),
                &NFT_CONTRACT_ACCOUNT,
                0,
                BASE_CALL_GAS,
            )
            .then(ext_self::distribute_rewards(
                nonce,
                &env::current_account_id(),
                env::attached_deposit(),
                4 * BASE_CALL_GAS,
            ));
        }
    }

    #[allow(dead_code)]
    #[payable]
    pub fn distribute_rewards(&mut self, nonce: String) {
        let campaign_promise_res =
            promise_result_as_success().expect("nft_token(method) callback: promise failed");
        let campaign = near_sdk::serde_json::from_slice::<JsonToken>(&campaign_promise_res)
            .expect("Not valid NFT campaign");

        let mut updated_metadata = campaign.metadata.clone();
        updated_metadata.update_no = Some(U64(u64::from(updated_metadata.update_no.unwrap()) + 1));

        // Reward user after his campaign has been verified, and update nft.
        ext_contract::ft_transfer_from(
            campaign.owner_id,
            self.nonce_owner.get(&nonce).unwrap(),
            U128(
                u128::from(campaign.metadata.ft_amount.unwrap())
                    / u64::from(campaign.metadata.max_updates.unwrap()) as u128,
            ),
            None,
            &FT_CONTRACT_ACCOUNT,
            env::attached_deposit(),
            BASE_CALL_GAS,
        )
        .then(ext_contract::nft_update(
            campaign.token_id,
            updated_metadata,
            &NFT_CONTRACT_ACCOUNT,
            0,
            BASE_CALL_GAS,
        ));
    }

    #[allow(dead_code)]
    pub fn get_received_vals(&self, max: U128) -> HashMap<String, OracleCallback> {
        let mut counter: u128 = 0;
        let mut result: HashMap<String, OracleCallback> = HashMap::new();
        for answer in self.received.iter() {
            if counter == max.0 || counter > self.received.len() as u128 {
                break;
            }
            result.insert(answer.0.to_string(), answer.1);
            counter += 1;
        }
        result
    }
}
