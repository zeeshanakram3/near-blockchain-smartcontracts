use super::*;
const MAX_RESERVE_RATIO: f64 = 1.0;
pub trait BondingCurveInterface {
    fn calculate_purchase_return(
        &mut self,
        token_supply: Balance,
        reserve_balance: Balance,
        reserve_ratio: f64,
        deposit_amount: Balance,
    ) -> Balance;
    fn calculate_sale_return(
        &mut self,
        token_supply: Balance,
        reserve_balance: Balance,
        reserve_ratio: f64,
        sell_amount: Balance,
    ) -> Balance;
}

impl BondingCurveInterface for SocialTokens {
    fn calculate_purchase_return(
        &mut self,
        token_supply: Balance,
        reserve_balance: Balance,
        reserve_ratio: f64,
        deposit_amount: Balance,
    ) -> Balance {
        if reserve_ratio > MAX_RESERVE_RATIO {
            env::panic(b"Invalid inputs to calculate purchase return")
        }
        if deposit_amount == 0 {
            return 0;
        }
        if reserve_ratio == MAX_RESERVE_RATIO {
            return token_supply * deposit_amount / reserve_balance;
        }
        let buy_amount = token_supply as f64
            * ((1.0 + (deposit_amount as f64 / reserve_balance as f64))
                .powf(reserve_ratio / MAX_RESERVE_RATIO)
                - 1.0);
        buy_amount.ceil() as u128
    }

    fn calculate_sale_return(
        &mut self,
        token_supply: Balance,
        reserve_balance: Balance,
        reserve_ratio: f64,
        sell_amount: Balance,
    ) -> Balance {
        if reserve_ratio > MAX_RESERVE_RATIO {
            env::panic(b"Invalid inputs to calculate purchase return")
        }
        if sell_amount == 0 {
            return 0;
        }
        if sell_amount == token_supply {
            return reserve_balance;
        }

        if reserve_ratio == MAX_RESERVE_RATIO {
            return reserve_balance * sell_amount / token_supply;
        }

        let return_amount = reserve_balance as f64
            * ((1.0 - (sell_amount as f64 / token_supply as f64)).powf(1.0 / reserve_ratio) - 1.0);
        return_amount.ceil().abs() as u128
    }
}
