use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{ensure, Deps};

use crate::{
    helpers::{balance::to_asset_info, route::reversed}, 
    state::{
        common::key_from, pools::get_pool, routes::find_route
    }, 
    types::{
        pair::{Pair, PopulatedPair}, 
        pool::{Pool, PopulatedPool}, 
        route::{PopulatedRoute, Route, RouteHop}
    }, 
    ContractError
};

use super::populated::populated_pool;


/// pair validation when creating a new pair
pub fn validated_pair_on_creation(
    deps:        Deps, 
    pair:        &Pair, 
) -> Result<PopulatedPair, ContractError> {
    
    if pair.is_pool_pair() {
        validated_pool_pair_on_creation(deps, pair)
    } else {
        validated_routed_pair_on_creation(deps, pair)
    }
}


/// a helper validating that the pool of a direct pair is valind and convert it to new PopulatedPair
fn validated_pool_pair_on_creation(deps: Deps, pair: &Pair) -> Result<PopulatedPair, ContractError> {
    Ok(validated_pool(deps, &pair.pool(), false)?.into())
}


/// a helper validation that the route of a routed pair is valid and convert it to new PopulatedPair
fn validated_routed_pair_on_creation(
    deps:        Deps,
    pair:        &Pair,
) -> Result<PopulatedPair, ContractError> {
    Ok(validated_route(deps, &pair.base_asset, &pair.quote_asset, &pair.route())?.into())
}


/// used for both validation a newly created pool pair and for checking that route can be reconstructed
fn validated_pool(deps: Deps, pool: &Pool, allow_storage: bool) -> Result<PopulatedPool, ContractError> {
    if allow_storage {
        let from_storage = get_pool(deps.storage, key_from(&pool.denoms()));
        if from_storage.is_ok() {
            return Ok(from_storage?.into());
        }
    }
    populated_pool(&deps.querier, pool)
}



pub fn validated_routed_pair(
    deps:        Deps,
    pair:        &Pair,
    offer_asset: Option<AssetInfo>
) -> Result<PopulatedPair, ContractError> {

    let offer_asset = offer_asset.unwrap_or(pair.base_asset.clone());

    let (
        base, quote, reverse
    ) = if offer_asset.equal(&pair.base_asset) {
        (&pair.base_asset, &pair.quote_asset, false)
    } else {
        ensure!(offer_asset.equal(&pair.quote_asset), ContractError::RouteNotFound{});
        (&pair.quote_asset, &pair.base_asset, true)
    };

    let from_storage = find_route(
        deps.storage, 
        [base.to_string(), quote.to_string()],
        reverse
    );

    if from_storage.is_ok() {
        return Ok(from_storage?.into());
    }

    let route = if reverse {
        reversed(&pair.route())
    } else {
        pair.route()
    };
    
    Ok(validated_route(deps, base, quote, &route)?.into())
}



pub fn validated_route(
    deps:        Deps,
    base_asset:  &AssetInfo,
    quote_asset: &AssetInfo,
    route:       &Route,
) -> Result<PopulatedRoute, ContractError> {
    ensure!(route.len() > 0, ContractError::RouteEmpty{});
    ensure!(!base_asset.equal(quote_asset), ContractError::SameAsset{});

    let mut route_denoms : Vec<String> = Vec::with_capacity(route.len() + 2);
    route_denoms.push(base_asset.to_string()); 

    let mut hops = Vec::with_capacity(route.len() + 1);
    let first = route.first().unwrap();
    ensure!(first.prev.asset_info.equal(base_asset), ContractError::InvalidHops{});
    hops.push(validated_hop(deps, &first, false)?);

    for (index, hop) in route.iter().enumerate().skip(1) {
        let prev_denom = &route.get(index - 1).clone().unwrap().denom;
        ensure!(*prev_denom == hop.prev.asset_info.to_string(), ContractError::InvalidHops{});
        hops.push(validated_hop(deps, &hop, false)?);
        route_denoms.push(hop.denom.clone());
    }

    let last = route.last().unwrap();
    ensure!(last.next.is_some(), ContractError::MissingNextPoolHop{});
    let next = last.next.clone().unwrap();
    ensure!(next.asset_info.equal(quote_asset), ContractError::InvalidHops{});
    hops.push(validated_hop(deps, &last, true)?);
    route_denoms.push(quote_asset.to_string());

    validate_route_denoms(&route_denoms)?;

    Ok(hops)
}



fn validated_hop(
    deps:       Deps,
    hop:        &RouteHop,
    next:       bool
) -> Result<PopulatedPool, ContractError> {
    let hop_asset = to_asset_info(&hop.denom);
    let (hop_pool, base_asset, quote_asset) = if next {
        let hop_pool = hop.next.clone().unwrap();
        (hop_pool.clone(), hop_asset, hop_pool.asset_info)
    } else {
        (hop.prev.clone(), hop.prev.asset_info.clone(), hop_asset)
    };
    let pool = Pool {
        address: hop_pool.address,
        pool_type: hop_pool.pool_type,
        base_asset,
        quote_asset,
    };
    validated_pool(deps, &pool, true)
}




fn validate_route_denoms(denoms: &Vec<String>) -> Result<(), ContractError> {
    let mut denoms_set = denoms.clone();
    denoms_set.sort();
    denoms_set.dedup();
    if denoms_set.len() != denoms.len() {
        return Err(ContractError::RouteDublicates { });
    }
    Ok(())
}
