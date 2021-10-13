use super::*;
use near_sdk::serde::Serialize;

#[derive(Serialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SocialTokensMetadata {
    pub version: Option<String>,
    pub name: String,
    pub reference: Option<String>,
    pub reference_hash: Option<[u8; 32]>,
}

pub trait SocialTokensMetadataProvider {
    fn st_metadata(&self) -> SocialTokensMetadata;
}

#[near_bindgen]
impl SocialTokensMetadataProvider for SocialTokens {
    fn st_metadata(&self) -> SocialTokensMetadata {
        self.st_metadata.clone()
    }
}
