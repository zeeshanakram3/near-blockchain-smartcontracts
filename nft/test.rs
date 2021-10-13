// use the attribute below for unit tests
use super::*;
use std::convert::TryFrom;

use near_sdk::MockedBlockchain;
use near_sdk::{testing_env, VMContext};

fn robert() -> AccountId {
    "robert.testnet".to_string()
}

// part of writing unit tests is setting up a mock context
// this is a useful list to peek at when wondering what's available in env::*
fn get_context(
    predecessor_account_id: String,
    storage_usage: u64,
    attached_deposit: Balance,
) -> VMContext {
    VMContext {
        current_account_id: ".testnet".to_string(),
        signer_account_id: "jane.testnet".to_string(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id,
        input: vec![],
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        storage_usage,
        attached_deposit,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view: false,
        output_data_receivers: vec![],
        epoch_height: 19,
    }
}

#[test]
fn init_nft_contract() {
    let context = get_context(robert(), 0, 0);
    testing_env!(context);

    let mut type_supply_caps = HashMap::new();
    type_supply_caps.insert("unique".to_string(), U64(1));
    let nft = NonFungibleToken::new(
        ValidAccountId::try_from(robert()).unwrap(),
        NFTMetadata {
            name: "test-nft".to_string(),
            spec: "test-spec".to_string(),
            symbol: "TEST".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        },
        type_supply_caps,
    );
    assert_eq!(nft.owner_id, robert(), "Invalid setters on nft.owner_id");
    let metadata = nft.metadata.get().unwrap();
    assert_eq!(
        metadata.name, "test-nft",
        "Invalid setters for metadata.name"
    );
    assert_eq!(
        metadata.symbol, "TEST",
        "Invalid setters for metadata.symbol"
    )
}

#[test]
fn mint_nft() {
    let storage_usage: u64 = 0;
    let context = get_context(robert(), storage_usage, 8790000000000000000000);
    testing_env!(context);

    let mut type_supply_caps = HashMap::new();
    type_supply_caps.insert("unique".to_string(), U64(1));

    let mut nft = NonFungibleToken::new(
        ValidAccountId::try_from(robert()).unwrap(),
        NFTMetadata {
            name: "test-nft".to_string(),
            spec: "test-spec".to_string(),
            symbol: "TEST".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        },
        type_supply_caps,
    );

    let token_metadata = TokenMetadata {
        title: Some("Best Nft of the World".to_string()), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        description: None,                                // free-form description
        media: None, // URL to associated media, preferably to decentralized, content-addressed storage
        media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
        max_updates: None, //max copies allowed  //// number of copies of this set of metadata in existence when token was minted.
        update_no: None,
        ft_account_id: None,
        ft_amount: None,
        issued_at: None,      // ISO 8601 datetime when token was issued or minted
        expires_at: None,     // ISO 8601 datetime when token expires
        starts_at: None,      // ISO 8601 datetime when token starts being valid
        updated_at: None,     // ISO 8601 datetime when token was last updated
        extra: None, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
        reference: None, // URL to an off-chain JSON file with more info.
        reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
    };
    let mint = nft.nft_mint(
        Some("1000".to_string()),
        token_metadata,
        None,
        None,
        None,
        None,
    );
    assert_eq!(mint, "1000".to_string(), "Invalid setters on mint.token_id");
    let nft_token = nft.tokens_by_id.get(&"1000".to_string()).unwrap();
    assert_eq!(
        nft_token.owner_id,
        robert(),
        "Invalid setter on nft.owner_id"
    )
}
