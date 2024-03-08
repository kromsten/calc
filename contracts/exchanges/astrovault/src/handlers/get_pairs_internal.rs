use cosmwasm_std::{Deps, StdResult};

use crate::{
    state::pairs::{get_pairs, get_pairs_full},
    types::pair::{Pair, PopulatedPair},
};

pub fn get_pairs_internal_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<Pair>> {
    get_pairs(
        deps.storage,
        start_after.map(|pair| pair.denoms()),
        limit,
    )
}

pub fn get_pairs_internal_full_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<PopulatedPair>> {
    get_pairs_full(
        deps.storage,
        start_after.map(|pair| pair.denoms()),
        limit,
    )
}
