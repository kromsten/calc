use cosmwasm_std::{Deps, StdResult};

use crate::{state::pairs::get_pairs, types::pair::Pair};

pub fn get_pairs_internal_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<Pair>> {
    Ok(get_pairs(
        deps.storage,
        start_after.map(|pair| [pair.base_denom, pair.quote_denom]),
        limit,
    ))
}
