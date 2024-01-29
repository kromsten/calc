use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{ensure, Deps};

use crate::{
    helpers::balance::to_asset_info, state::{pairs::find_pair, pools::find_pool, routes::find_route}, types::{pair::{Pair, PopulatedPair}, pool::{Pool, PopulatedPool}, route::{HopInfo, PopulatedRoute, Route}}, ContractError};

use super::populated::populated_pool;




pub fn validated_pair(
    deps:        Deps, 
    pair:        &Pair, 
    offer_asset: Option<AssetInfo>
) -> Result<PopulatedPair, ContractError> {
    let from_storage = find_pair(deps.storage, pair.denoms());
    if from_storage.is_ok() {
        Ok(from_storage?)
    } else {
        if pair.is_pool_pair() {
            validated_pool_pair(deps, pair)
        } else {
            validated_routed_pair(deps, pair, offer_asset)
        }
    }
}


pub fn validated_pool(deps: Deps, pool: &Pool) -> Result<PopulatedPool, ContractError> {
    let from_storage = find_pool(deps.storage, pool.denoms());
    if from_storage.is_ok() {
        Ok(from_storage?)
    } else {
        populated_pool(&deps.querier, pool)
    }
}

pub fn validated_pool_pair(deps: Deps, pair: &Pair) -> Result<PopulatedPair, ContractError> {
    Ok(validated_pool(deps, &pair.pool())?.into())
}


pub fn validated_hop_info(
    deps:       Deps,
    hop_info:   &HopInfo,
    other:      &AssetInfo,
) -> Result<PopulatedPool, ContractError> {

    let pool = Pool {
        address: hop_info.address.clone(),
        base_asset: other.clone(),
        quote_asset: hop_info.asset_info.clone(),
        pool_type: hop_info.pool_type.clone(),
    };

    validated_pool(deps, &pool)
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



pub fn validated_route(
    deps:        Deps,
    base_asset:  &AssetInfo,
    quote_asset: &AssetInfo,
    route:       &Route,
) -> Result<PopulatedRoute, ContractError> {
    ensure!(route.len() > 0, ContractError::RouteEmpty{});

    let mut route_denoms : Vec<String> = Vec::with_capacity(route.len() + 2);
    route_denoms.push(base_asset.to_string()); 

    let mut hops = Vec::with_capacity(route.len() + 1);
    let first = route.first().unwrap();
    ensure!(first.prev.asset_info.equal(base_asset), ContractError::InvalidHops{});
    hops.push(validated_hop_info(deps, &first.prev, &base_asset)?);


    for (index, hop) in route.iter().enumerate().skip(1) {
        let prev_denom = &route.get(index - 1).clone().unwrap().denom;
        ensure!(*prev_denom == hop.prev.asset_info.to_string(), ContractError::InvalidHops{});
        hops.push(validated_hop_info(deps, &hop.prev, &to_asset_info(prev_denom))?);
        route_denoms.push(hop.denom.clone());
    }
    

    let last = route.last().unwrap();
    ensure!(last.next.is_some(), ContractError::MissingNextPoolHop{});
    let next = last.next.clone().unwrap();
    ensure!(next.asset_info.equal(quote_asset), ContractError::InvalidHops{});
    hops.push(validated_hop_info(deps, &next, &quote_asset)?);
    route_denoms.push(quote_asset.to_string());

    validate_route_denoms(&route_denoms)?;

    Ok(hops)
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

    let mut route = pair.route();
    if reverse {
        route.reverse();
    }

    Ok(validated_route(deps, base, quote, &route)?.into())
}

