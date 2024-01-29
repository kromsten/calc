use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Map;

use crate::{helpers::route::populated_route_denoms, state::{common::sorted_denoms, pools::get_pool}, types::{pair::PopulatedPair, pool::PopulatedPool, route::{PopulatedRoute, StoredRoute}}, ContractError}
;

use super::{common::{allow_implicit, key_from, PAIRS}, pools::save_pool};



/// list if intermidiate denoms between base and quote assets
/// sorted (base.denom, quote.denom) -> List of denoms where e
/// ach two consecutive denoms are pools that exist in storage
const ROUTES          : Map<String, StoredRoute>   = Map::new("sr_v1");




pub fn route_exists(
    storage:    &dyn Storage, 
    denoms:     &[String; 2],
) -> bool {
    ROUTES.has(storage, key_from(denoms))
}



fn get_stored_route(
    storage:        &dyn Storage, 
    key:            String,
    reverse:        bool
) -> StdResult<StoredRoute> {
    let mut route = ROUTES.load(storage, key)?;
    if reverse {
        route.reverse();
    }
    Ok(route)
}


pub fn get_route(
    storage:        &dyn Storage, 
    denoms:         [String; 2],
    reverse:        bool
) -> StdResult<PopulatedRoute> {
    let sorted = sorted_denoms(&denoms);
    
    let [base, quote] = if reverse {
        [sorted[1].clone(), sorted[0].clone()]
    } else {
        sorted
    };

    let key = format!("{}-{}", base, quote);

    let route = get_stored_route(storage, key.clone(), reverse)?;

    let mut hop_pools : Vec<PopulatedPool> = Vec::with_capacity(route.len() + 2);

    let first = route.first().unwrap().clone();
    let last = route.last().unwrap().clone();

    hop_pools.push(
        get_pool(storage, key_from(&[base, first]))?
    );


    for (index, denom) in route.iter().skip(1).enumerate() {
        let prev = route.get(index).unwrap().clone();
        hop_pools.push(
            get_pool(storage, key_from(&[denom.clone(), prev]))?
        );
    }


    hop_pools.push(
        get_pool(storage, key_from(&[last, quote]))?
    );

    
    Ok(hop_pools)
}



pub fn get_routed_pair(
    storage:        &dyn Storage, 
    denoms:         [String; 2],
    reverse:        bool
) -> StdResult<PopulatedPair> {
    Ok(get_route(storage, denoms, reverse)?.into())
}



pub fn save_routed_pair(
    storage:        &mut dyn Storage,
    pair:           &PopulatedPair
) -> Result<(), ContractError> {

    let [base, quote] = sorted_denoms(&pair.denoms());
    let key =  format!("{}-{}", base, quote);

    PAIRS.save(storage, key.clone(), &pair.into())?;

    
    let route = pair.route();

    route
    .iter()
    .map(|pool| save_pool(storage, key_from(&pool.denoms()), pool))
    .collect::<Result<Vec<()>, ContractError>>()?;


    let denoms = populated_route_denoms(&route);
    for (base_index, base) in denoms.iter().enumerate() {

        for (quote_index, quote) in denoms.iter().enumerate().skip(base_index + 2) {
            let between = &denoms[base_index + 1..quote_index].to_vec();
            ROUTES.save(storage, key_from(&[base.clone(), quote.clone()]), between)?;
        }

    }
    Ok(())
}



pub fn delete_routed_pair(
    storage: &mut dyn Storage,
    pair:    &PopulatedPair
) {
    let key = key_from(&pair.denoms());
    PAIRS.remove(storage, key.clone());
    ROUTES.remove(storage, key)
}




pub fn find_route(
    storage:    &dyn Storage, 
    denoms:     [String; 2],
    reverse:    bool
) -> StdResult<PopulatedRoute> {

    let pair = PAIRS.load(storage, key_from(&denoms));
    
    if pair.is_ok() {
        return get_route(storage, denoms, reverse);
    }

    let routed = get_route(storage, denoms, reverse);

    if allow_implicit(storage) {
        routed
        
    } else {
        Err(StdError::generic_err(
            if routed.is_ok() {
                "Routed pair exist but implicit pairs are not allowed"
            } else {
                "Routed pair is not found"
            }
        ))
    }
}




