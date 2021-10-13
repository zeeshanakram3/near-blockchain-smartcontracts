use crate::*;

#[near_bindgen]
impl SocialTokens {
    pub(crate) fn st_collateral_account_id(&self, token_symbol: Symbol) -> ValidAccountId {
        let symbol_hash = env::sha256(token_symbol.as_bytes());
        self.social_tokens
            .get(&symbol_hash)
            .expect("No social token exists for given symbol")
            .collateral_account_id
    }

    pub(crate) fn get_social_token(&self, token_symbol: Symbol) -> Option<SocialToken> {
        let symbol_hash = env::sha256(token_symbol.as_bytes());
        self.social_tokens.get(&symbol_hash)
    }

    pub(crate) fn set_social_token(&mut self, token_symbol: Symbol, social_token: &SocialToken) {
        let symbol_hash = env::sha256(token_symbol.as_bytes());
        self.social_tokens.insert(&symbol_hash, social_token);
    }

    pub(crate) fn st_get_balance(
        &self,
        token_symbol: Symbol,
        account_id: &AccountId,
    ) -> Option<Balance> {
        let owned_string = token_symbol.to_owned();
        let symbol_account_id_hash = env::sha256(owned_string.add(account_id).as_bytes());
        self.st_to_accounts.get(&symbol_account_id_hash)
    }

    pub(crate) fn st_set_balance(
        &mut self,
        token_symbol: Symbol,
        account_id: &AccountId,
        balance: Balance,
    ) {
        let owned_string = token_symbol.to_owned();
        let symbol_account_id_hash = env::sha256(owned_string.add(account_id).as_bytes());
        self.st_to_accounts
            .insert(&symbol_account_id_hash, &balance);
    }

    pub(crate) fn continuous_mint(&mut self, collateral_amount: Balance, token_symbol: String) {
        let mut social_token = match self.get_social_token(token_symbol.clone()) {
            Some(st) => st,
            None => env::panic(b"Social token with given symbol does not exist."),
        };
        social_token.collateral_amount += collateral_amount;
        let tokens_bought = self.calculate_purchase_return(
            social_token.supply,
            social_token.collateral_amount.into(),
            0.1,
            collateral_amount,
        );
        social_token.supply += tokens_bought;
        self.set_social_token(token_symbol.clone(), &social_token);
        let mut balance = self
            .st_get_balance(token_symbol.clone(), &env::predecessor_account_id())
            .expect(&(env::predecessor_account_id().clone() + " account is not registered"));
        balance += tokens_bought;
        self.st_set_balance(token_symbol, &env::predecessor_account_id(), balance);
    }

    pub(crate) fn continuous_burn(&mut self, token_amount: Balance, token_symbol: String) {
        let mut social_token = match self.get_social_token(token_symbol.clone()) {
            Some(st) => st,
            None => env::panic(b"Social token with given symbol does not exist."),
        };
        social_token.supply -= token_amount;
        let collateral_return = self.calculate_sale_return(
            social_token.supply,
            social_token.collateral_amount,
            0.1,
            token_amount,
        );
        social_token.collateral_amount -= collateral_return;
        self.set_social_token(token_symbol.clone(), &social_token);

        let mut balance = self
            .st_get_balance(token_symbol.clone(), &env::predecessor_account_id())
            .expect(&(env::predecessor_account_id().clone() + " account is not registered"));
        balance -= token_amount;
        self.st_set_balance(token_symbol, &env::predecessor_account_id(), balance);
    }

    pub(crate) fn internal_deposit(
        &mut self,
        token_symbol: Symbol,
        account_id: &AccountId,
        amount: Balance,
    ) {
        let balance = self
            .st_get_balance(token_symbol.clone(), account_id)
            .expect(&(env::predecessor_account_id().clone() + " account is not registered"));
        if let Some(new_balance) = balance.checked_add(amount) {
            self.st_set_balance(token_symbol, account_id, new_balance);
        } else {
            env::panic(b"Balance overflow");
        }
    }

    pub(crate) fn internal_withdraw(
        &mut self,
        token_symbol: Symbol,
        account_id: &AccountId,
        amount: Balance,
    ) {
        let balance = self
            .st_get_balance(token_symbol.clone(), account_id)
            .expect(&(env::predecessor_account_id().clone() + " account is not registered"));
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.st_set_balance(token_symbol, account_id, new_balance);
        } else {
            env::panic(b"The account doesn't have enough balance");
        }
    }

    pub(crate) fn internal_transfer(
        &mut self,
        token_symbol: Symbol,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );
        self.internal_withdraw(token_symbol.clone(), sender_id, amount);
        self.internal_deposit(token_symbol, receiver_id, amount);
        env::log(format!("Transfer {} from {} to {}", amount, sender_id, receiver_id).as_bytes());
        if let Some(memo) = memo {
            env::log(format!("Memo: {}", memo).as_bytes());
        }
    }
}
