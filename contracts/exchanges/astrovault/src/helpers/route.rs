#![allow(unused_variables, unused_imports)]

use crate::{state::pairs::pair_is_stored, types::{pair::{Pair, PoolInfo}, position_type::PositionType}, ContractError};
use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{from_json, Deps, QuerierWrapper, StdError, StdResult};
use super::{pair, pool};


fn validated_route_pool(
    deps:               Deps,
    pool:               &PoolInfo,
) -> Result<PoolInfo, ContractError> {
    let mut pool = pool.clone();
    pool.populate(&deps.querier)?;
    pool.validate(deps)?;
    Ok(pool)
}


fn validated_route_pair(
    deps:               Deps,
    pair:               &Pair,
    allow_missing:      bool,
) -> Result<Pair, ContractError> {

    if !allow_missing && !pair_is_stored(deps.storage, &pair) {
        return Err(ContractError::InvalidRoute {
            base: pair.base_denom(),
            quote: pair.quote_denom(),
        });
    };
    let pool = validated_route_pool(deps, &pair.pool_info())?;
    Ok(pool.into())
}

/// simple check that the route is valid without returning anything
pub fn validate_routed_pair(
    deps:                   Deps,
    pair:                   &Pair,
) -> Result<(), ContractError> {
    pair
    .route_pools()
    .iter()
    .map(|pool| validated_route_pool(deps, pool))
    .collect::<Result<Vec<PoolInfo>, ContractError>>()?;
    Ok(())
}


/// check that the route is valid, populate indexes and return a list of (pool) pairs to be saved
pub fn validated_route_pairs(
    deps:                   Deps,
    pair:                   &Pair,
    allow_missing:          bool,
) -> Result<Vec<Pair>, ContractError> {
    pair
    .route_pairs()
    .iter()
    .map(|pair| validated_route_pair(deps, pair, allow_missing))
    .collect::<Result<Vec<Pair>, ContractError>>()
}



pub fn get_token_out_denom(
    querier: &QuerierWrapper,
    token_in_denom: String,
    pool_id: u64,
    next_pool_id: u64,
) -> StdResult<String> {
    todo!()
}


pub fn get_pool_assets(querier: &QuerierWrapper, pool_id: u64) -> Result<Vec<String>, StdError> {
    todo!()
}

pub fn calculate_route(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_denom: String,
) -> StdResult<Vec<()>> {
    todo!()
}


/* 
#[cfg(test)]
mod get_token_out_denom_tests {

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        todo!()
    }
}

#[cfg(test)]
mod calculate_route_tests {
    use super::calculate_route;

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()

    }

    #[test]
    fn when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn calculates_2_pool_route() {
        
        todo!()

    }

    #[test]
    fn calculates_3_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_calculates_2_pool_route() {
        todo!()
    }

    #[test]
    fn with_cl_pools_calculates_3_pool_route() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_calculates_2_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_calculates_3_pool_route() {
        
        todo!()
    }
}
 */