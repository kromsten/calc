#![allow(unused_variables, unused_imports)]

use crate::{state::{config::{get_config, get_router_config}, pairs::{find_route_pair, pair_exists, route_pair_exists}}, types::{pair::{Pair, PoolInfo}, position_type::PositionType, wrapper::ContractWrapper}, ContractError};
use astrovault::assets::asset::{Asset, AssetInfo};
use cosmwasm_std::{ensure, from_json, to_json_binary, Binary, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, StdResult, Uint128};
use cw20::Cw20ReceiveMsg;
use super::{pair, pool::{self, validated_direct_pair}};
use astrovault::router::{
    state::{
        Hop as AstroHop, 
        Route as AstroRoute
    },
    handle_msg::ExecuteMsg as RouterExecute
};



fn validated_route_pair(
    deps:               Deps,
    pair:               &Pair,
    allow_missing:      bool,
) -> Result<(Pair, bool), ContractError> {
    let found =  find_route_pair(deps.storage, pair.denoms());
    
    match found {
        Ok(pair) => Ok((pair, true)), 
        Err(_) => {
            if allow_missing {
                return Ok((validated_direct_pair(deps, pair)?, false));
            } else {
                return Err(ContractError::NoRoutedPair {});
            }
        }
    }
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
    .map(|pair| {
        let val = validated_route_pair(deps, pair, allow_missing)?;
        Ok(val.0)
    })
    .collect::<Result<Vec<Pair>, ContractError>>()
}



pub fn validated_route_pairs_to_save(
    deps:                   Deps,
    pair:                   &Pair,
) -> Result<Vec<Pair>, ContractError> {
    pair
    .route_pairs()
    .iter()
    .map(|pair| validated_route_pair(deps, pair, true))
    .filter_map(|res| {
        match res {
            Ok((pair, existed)) => {
                if existed {
                    None
                } else {
                    Some(Ok(pair))
                }
            },
            Err(err) => Some(Err(err))
        }
    })
    .collect::<Result<Vec<Pair>, ContractError>>()
}


pub fn route_pairs_to_astro_hops(
    deps:           Deps,
    pair_hops:      &Vec<Pair>,
    offer_asset:    &Asset,
    target_info:    &AssetInfo,
) -> Result<Vec<AstroHop>, ContractError> {
    let mut astro_hops: Vec<AstroHop> = Vec::with_capacity(pair_hops.len());

    let first = pair_hops.first().unwrap();
    let last = pair_hops.last().unwrap();

    let mut offer_asset = offer_asset.info.clone();

    ensure!(first.base_asset == offer_asset || 
            first.quote_asset == offer_asset, ContractError::RouteRuntimeError {});


    for hop_pair in pair_hops {
        let astro_hop = hop_pair.to_astro_hop(&deps.querier, &offer_asset)?;
        astro_hops.push(astro_hop);

        offer_asset = hop_pair.other_asset(&offer_asset);
        
        if hop_pair.eq(last) {
            ensure!(offer_asset == *target_info, ContractError::RouteRuntimeError {});
        }
    }

    Ok(astro_hops)
}



pub fn route_swap_cosmos_msg(
    deps:                Deps,
    env:                 Env,
    pair:                &Pair,
    offer_asset:         Asset,
    target_asset:        Asset,
    route:               Option<Binary>,
    funds:               Vec<Coin>,
) -> Result<CosmosMsg, ContractError> {

    let mut hops_pairs = pair.route_pairs()
                            .iter()
                            .map(|pair| validated_route_pair(deps, pair, false))
                            .map(|res| Ok(res?.0))
                            .collect::<Result<Vec<Pair>, ContractError>>()?;

    if offer_asset.info == pair.quote_asset {
        hops_pairs.reverse();
    }

    let hops = route_pairs_to_astro_hops(
        deps,
        &hops_pairs,
        &offer_asset,
        &target_asset.info,
    )?;

    let hop_binary = to_json_binary(&hops)?;

    let route = AstroRoute {
        hops,
        minimum_receive: Some(target_asset.amount),
        to: None,
    };


    let cfg = get_config(deps.storage)?;
    
    let router = ContractWrapper(cfg.router_address.into());

    let msg = if offer_asset.info.is_native_token() {
        router.execute(
            to_json_binary(&RouterExecute::Receive(Cw20ReceiveMsg {
                sender: env.contract.address.to_string(),
                amount: offer_asset.amount,
                msg: hop_binary,
            }))?, 
            funds
        )?
    } else {
        router.execute_cw20(
            offer_asset.to_string(), 
            offer_asset.amount, 
            hop_binary
        )?
    };

    Ok(msg)

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