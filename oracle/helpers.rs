use crate::*;

#[near_bindgen]
impl Oracle {
    pub fn get_nonce(&self, account: AccountId) -> Option<Option<U128>> {
        self.nonces.get(&account)
    }

    pub fn get_nonces(&self) -> HashMap<AccountId, Option<U128>> {
        let mut result: HashMap<AccountId, Option<U128>> = HashMap::new();
        for nonce in self.nonces.iter() {
            result.insert(nonce.0.clone(), nonce.1.clone());
        }
        result
    }
}

impl Oracle {
    pub fn reset(&mut self) {
        self._only_owner();
        self.requests.clear();
        env::log(b"Commitments and requests are cleared.");
    }

    /// Can be called after a cross-contract call before enforcing a panic
    pub fn panic(&mut self, error_message: String) {
        self._only_owner_predecessor();
        env::panic(error_message.as_bytes());
    }

    pub fn _only_owner(&mut self) {
        assert_eq!(
            env::signer_account_id(),
            env::current_account_id(),
            "Only contract owner can call this method."
        );
    }

    /// This is a helper function with the promises happening.
    /// The predecessor will be this account calling itself after transferring
    /// fungible tokens. Used for functions called via promises where we
    /// do not want end user accounts calling them directly.
    pub fn _only_owner_predecessor(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only contract owner can sign transactions for this method."
        );
    }

    pub fn _only_authorized_node(&mut self) {
        assert!(
            self.authorized_nodes.contains(&env::signer_account_id())
                || env::signer_account_id() == env::current_account_id(),
            "Not an authorized node to fulfill requests."
        );
    }

    pub fn _check_callback_address(&mut self, callback_address: &AccountId) {
        assert_ne!(
            callback_address,
            &env::current_account_id(),
            "Callback address cannot be the oracle contract."
        );
    }
}
