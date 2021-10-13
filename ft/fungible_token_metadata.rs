use super::*;
use near_sdk::serde::Serialize;

#[derive(Serialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub version: Option<String>,
    pub name: String,
    pub symbol: String,
    pub reference: Option<String>,
    pub reference_hash: Option<[u8; 32]>,
    pub decimals: u8,
}

pub trait FungibleTokenMetadataProvider {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for FungibleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.ft_metadata.clone()
    }
}
