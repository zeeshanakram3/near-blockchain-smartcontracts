use super::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SocialToken {
    pub creator_id: AccountId,
    pub collateral_account_id: ValidAccountId,
    pub symbol: Symbol,
    pub collateral_amount: Balance,
    pub supply: Balance,
}

impl SocialToken {
    pub fn new(owner_id: AccountId, symbol: Symbol) -> Self {
        Promise::new(symbol.clone() + "." + ST_ACCOUNT_ID)
            .create_account()
            .as_return();
        Self {
            // TODO Ask use to submit 1 token as collateral
            creator_id: owner_id.to_string(),
            collateral_account_id: ValidAccountId::try_from(symbol.clone() + "." + ST_ACCOUNT_ID)
                .expect("collateral account does not exist. Maybe failed during creation"),
            symbol,
            collateral_amount: 0,
            supply: 0,
        }
    }
}
