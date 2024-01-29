use cosmwasm_std::{Deps, StdResult};
use exchange::msg::Pair;

use crate::state::pairs::get_pairs;

pub fn get_pairs_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<Pair>> {
    let pairs = get_pairs(deps.storage, start_after.map(|pair| pair.denoms), limit);
    
    Ok(pairs.into_iter().map(|pair| pair.into()).collect())
}
