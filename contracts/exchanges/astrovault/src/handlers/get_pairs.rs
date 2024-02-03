use cosmwasm_std::{Deps, StdResult};
use exchange::msg::Pair;

use crate::state::pairs::get_exchange_pairs;

pub fn get_pairs_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<Pair>> {
    Ok(get_exchange_pairs(
        deps.storage, 
        start_after.map(|pair| pair.denoms), 
        limit
    ))
}
