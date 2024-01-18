#![allow(unused_variables, unused_imports)]

use crate::types::{pair::Pair, position_type::PositionType};
use cosmwasm_std::{from_json, QuerierWrapper, StdError, StdResult};

pub fn get_token_out_denom(
    querier: &QuerierWrapper,
    token_in_denom: String,
    pool_id: u64,
    next_pool_id: u64,
) -> StdResult<String> {
    let pool_assets = get_pool_assets(querier, pool_id)?;

    if !pool_assets.contains(&token_in_denom) {
        return Err(StdError::generic_err(format!(
            "denom {} not found in pool id {}",
            token_in_denom, pool_id
        )));
    }

    let next_pool_assets = get_pool_assets(querier, next_pool_id)?;

    let intersecting_assets = pool_assets
        .iter()
        .filter(|asset| next_pool_assets.contains(*asset))
        .collect::<Vec<&String>>();

    if intersecting_assets.is_empty() {
        return Err(StdError::generic_err(format!(
            "pool {} contains no assets of the pool {}",
            next_pool_id, pool_id
        )));
    }

    Ok(intersecting_assets[0].clone())
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
