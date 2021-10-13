use super::*;

use near_sdk::{MockedBlockchain, Balance};
use near_sdk::{testing_env, VMContext};

#[allow(dead_code)]
fn robert() -> AccountId {
  "robert.testnet".to_string()
}

#[allow(dead_code)]
fn get_context(predecessor_account_id: String, storage_usage: u64, attached_deposit: Balance) -> VMContext {
  VMContext {
      current_account_id: ".testnet".to_string(),
      signer_account_id: robert(),
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
fn init() {
  let context = get_context(robert(), 0, 0);
  testing_env!(context);

  let contract = UsecasesContract::new(robert());
  assert_eq!(contract.owner_id, robert(), "Invalid setters on contract.order_id");
  assert_eq!(contract.request_nonce, 0, "Contract's initial nonce should be 0");
}

#[test]
fn add_oracle() {
  let context = get_context(robert(), 0, 0);
  testing_env!(context);

  let mut contract = UsecasesContract::new(robert());
  let account_id = ValidAccountId::try_from("oracle.testnet".to_string()).unwrap();
  contract.add_oracle(account_id);
  let added = contract.trusted_oracles.contains(&"oracle.testnet".to_string());
  assert_eq!(added, true, "Invalid oracle add on contract")
}

#[test]
fn remove_oracle() {
  let context = get_context(robert(), 0, 0);
  testing_env!(context);

  let mut contract = UsecasesContract::new(robert());
  let account_id = ValidAccountId::try_from("oracle.testnet".to_string()).unwrap();
  
  contract.add_oracle(account_id.clone());
  let mut has_oracle_contract = contract.trusted_oracles.contains(&"oracle.testnet".to_string());
  assert_eq!(has_oracle_contract, true, "Invalid oracle add on contract");

  contract.remove_oracle( account_id.into());
  has_oracle_contract = contract.trusted_oracles.contains(&"oracle.testnet".to_string());
  assert_eq!(has_oracle_contract, false, "Invalid oracle remove on contract")
}