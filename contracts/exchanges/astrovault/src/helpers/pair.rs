use cosmwasm_std::{ensure, Deps};
use crate::{types::pair::Pair, ContractError};
use crate::state::pairs::pair_is_stored;

#[cfg(not(test))]
use crate::helpers::pool::query_pool_exist;


pub fn pair_creatable(
    deps: Deps,
    pair: &Pair,
) -> Result<(), ContractError> {
    ensure!(!pair_is_stored(deps.storage, pair), ContractError::PairExist {});

    if pair.is_pool_pair() {
        #[cfg(not(test))]
        query_pool_exist(deps, &pair)?;
    } else {

    }


    Ok(())
}