use crate::*;
use std::vec::Vec;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CampaignsType(Vec<JsonToken>);

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub max_updates: Option<U64>, // max reward allowed/max recurring pass limit
    pub update_no: Option<U64>, // for `reward` nft, it will be no. of claimants who has recieved rewards,
    //  and for `subscrition` nft it will be no. of times recurring suscription pass has been updated.
    pub ft_account_id: Option<AccountId>,
    pub ft_amount: Option<U128>, // for `reward` nft, it will be reward amount.
    // For `subscription` nft, it will be pass price
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<Vec<String>>, // to store campaign operations
    pub reference: Option<String>, // to store tweet url
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CampaignVerificationResult {
    pub user_id: String,
    pub campaign_id: String,
    pub completed: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JSONCampaign {
    pub user_id: String,
    pub campaign_id: String,
    pub tweet_id: String,
    pub operations: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SubscriptionPassRequest {
    pub fulfill_at: String,
    pub subscription_pass_id: String,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: String,
    pub creator_id: ValidAccountId,
    pub owner_id: ValidAccountId,
    pub metadata: TokenMetadata,
    pub approved_account_ids: HashMap<AccountId, U64>,

    // CUSTOM - fields
    pub royalty: HashMap<AccountId, u32>,
    pub token_type: Option<String>,
}
