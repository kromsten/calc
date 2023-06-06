use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult};
use shared::coin::subtract;

pub fn get_balance_delta(
    querier: QuerierWrapper,
    address: Addr,
    old_balance: &Coin,
) -> StdResult<Coin> {
    let new_balance = querier.query_balance(address, old_balance.denom.clone())?;
    subtract(&new_balance, &old_balance)
}
