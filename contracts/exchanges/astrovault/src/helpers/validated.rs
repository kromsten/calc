use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{ensure, Deps};

use crate::{
    helpers::{balance::to_asset_info, route::reversed}, 
    state::{
        pairs::find_pair, 
        pools::find_pool, 
        routes::find_route
    }, 
    types::{
        pair::{Pair, PopulatedPair}, 
        pool::{Pool, PopulatedPool}, 
        route::{PopulatedRoute, Route, RouteHop}
    }, 
    ContractError
};

use super::populated::populated_pool;



pub fn validated_pair(
    deps:        Deps, 
    pair:        &Pair, 
    offer_asset: Option<AssetInfo>
) -> Result<PopulatedPair, ContractError> {
    let from_storage = find_pair(deps.storage, pair.denoms());
    if from_storage.is_ok() {
        let stored_pair = from_storage?;
        // only pool pair overriding is supported
        if pair.is_pool_pair() && stored_pair.is_pool_pair() {
            let pool = pair.pool();
            let mut stored_pool = stored_pair.pool();
            stored_pool.base_asset = pool.base_asset.clone();
            stored_pool.quote_asset = pool.quote_asset.clone();
            stored_pool.address = pool.address;
            Ok(stored_pool.into())
        } else {
            Ok(stored_pair)
        }
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


pub fn validated_hop(
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

    println!("validated_hop: {:?}", pool);

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
    println!("before first invalid check");
    ensure!(first.prev.asset_info.equal(base_asset), ContractError::InvalidHops{});
    hops.push(validated_hop(deps, &first, false)?);

    for (index, hop) in route.iter().enumerate().skip(1) {
        let prev_denom = &route.get(index - 1).clone().unwrap().denom;
        println!("before {} invalid check", index);
        ensure!(*prev_denom == hop.prev.asset_info.to_string(), ContractError::InvalidHops{});
        hops.push(validated_hop(deps, &hop, false)?);
        route_denoms.push(hop.denom.clone());
    }

    let last = route.last().unwrap();
    ensure!(last.next.is_some(), ContractError::MissingNextPoolHop{});
    let next = last.next.clone().unwrap();
    println!("before last invalid check");
    ensure!(next.asset_info.equal(quote_asset), ContractError::InvalidHops{});
    hops.push(validated_hop(deps, &last, true)?);
    route_denoms.push(quote_asset.to_string());

    validate_route_denoms(&route_denoms)?;

    Ok(hops)
}




pub fn validated_routed_pair(
    deps:        Deps,
    pair:        &Pair,
    offer_asset: Option<AssetInfo>
) -> Result<PopulatedPair, ContractError> {

    println!("\nbase: {:?}, quote: {:?}, offer: {:?}\n", pair.base_asset, pair.quote_asset, offer_asset);

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

