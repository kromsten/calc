use cosmwasm_std::{Order, StdError, StdResult, Storage};
use cw_storage_plus::Bound;

use crate::{state::{pools::pool_exists, routes::route_exists}, types::pair::{Pair, PopulatedPair, StoredPairType}, ContractError};

use super::{common::{allow_implicit, denoms_from, key_from, sorted_denoms, PAIRS}, pools::{find_pool, get_pool_pair, save_pool_pair, POOLS}, routes::{delete_routes_with_pool, find_route, get_routed_pair, save_routed_pair, ROUTES}};
use exchange::msg::Pair as ExchangePair;


/// Helper that check if explicily created pair exists
pub fn pair_exists(storage: &dyn Storage, denoms: &[String; 2]) -> bool {
    PAIRS.has(storage, key_from(&denoms))
}


pub fn find_pair(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<PopulatedPair> {

    let key = key_from(&denoms);
    let pair = PAIRS.load(storage, key.clone());
    
    if let Ok(stored) = pair {
        match stored {
            StoredPairType::Direct {} => get_pool_pair(storage, key),
            StoredPairType::Routed { } => get_routed_pair(storage, denoms, false)
        }

    }  else {

        if pool_exists(storage, &denoms) || route_exists(storage, &denoms) {

            if allow_implicit(storage) {

                if let Ok(pair) = get_pool_pair(storage, key) {
                    Ok(pair)
                } else if let Ok(pair) = get_routed_pair(storage, denoms, false) {
                    Ok(pair)
                } else {
                    Err(StdError::generic_err("Runtime error: couldn't get pair"))
                }
            } else {
                Err(StdError::generic_err("Pair exist but implicit pairs are not allowed"))
            }

        } else {
            Err(StdError::generic_err("Pair not found"))

        }

    }
}



pub fn find_pool_pair(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<PopulatedPair> {
    Ok(find_pool(storage, denoms)?.into())
}



pub fn find_route_pair(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<PopulatedPair> {
    let sorted = sorted_denoms(&denoms);
    let reverse = sorted[0] != denoms[0];
    Ok(find_route(storage, denoms, reverse)?.into())
}




pub fn save_pair(storage: &mut dyn Storage, pair: &PopulatedPair) -> Result<(), ContractError> {
    if pair.is_pool_pair() {
        save_pool_pair(storage, pair)
    } else {
        save_routed_pair(storage, pair)
    }
}


pub fn delete_pair(storage: &mut dyn Storage, pair: &PopulatedPair) {

    let key = key_from(&pair.denoms());
    PAIRS.remove(storage, key.clone());

    if pair.is_pool_pair() {
        POOLS.remove(storage, key.clone());
        delete_routes_with_pool(storage, key);
    } else {
        ROUTES.remove(storage, key);
    }
}


/// Returns simple [base, quote] denom pairs
/// Can be optimised a bit since it tries to populate all the pairs
/// first and then unpopulated them to the simplest form
pub fn get_exchange_pairs(
    storage: &dyn Storage,
    start_after: Option<[String; 2]>,
    limit: Option<u16>,
) -> Vec<ExchangePair> {
    get_pairs_full(
        storage,
        start_after,
        limit,
    )
    .into_iter()
    .map(|pair| pair.into())
    .collect()
}


/// Returns unpopulated pairs like they were originally provided
/// primarly for debugging and testing purposes
/// Can be optimised a bit since it first populate the pairs and then unpopulate them
pub fn get_pairs(
    storage: &dyn Storage,
    start_after: Option<[String; 2]>,
    limit: Option<u16>,
) -> Vec<Pair> {
    get_pairs_full(
        storage,
        start_after,
        limit,
    )
    .into_iter()
    .map(|pair| pair.into())
    .collect()
}


/// Returns populated pairs including pool routes and pools with asset indeces
/// Primarly for debuggins and testing purposes
/// The core function used by every other pair getter
/// Crashes and return nothing if any of the explicit pairs is not populatable
/// Skip implicit pairs if there was an error in their retrieval
pub fn get_pairs_full(
    storage: &dyn Storage,
    start_after: Option<[String; 2]>,
    limit: Option<u16>,
) -> Vec<PopulatedPair> {
    if allow_implicit(storage) {
        get_pairs_full_implicit(storage, start_after, limit)
    } else {
        PAIRS
        .range(
            storage,
            start_after.map(|denoms| Bound::exclusive(key_from(&denoms))),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(30) as usize)
        .flat_map(|result| 
            result.map(|(key, pair)| match pair {
                StoredPairType::Direct { } => find_pool_pair(storage, denoms_from(&key)),
                StoredPairType::Routed { } => find_route_pair(storage, denoms_from(&key))
            })
        )
        .collect::<StdResult<Vec<PopulatedPair>>>().unwrap()
    }

}


/// Returns fully populated information including pools and routes in the following order:
/// Both direct pools and routed pairs explicitly created by the admins. No specifc order internally
/// Direct pool pairs generated from hops of the explicitly created routed pairs
/// Routed pairs generated from inner routes of the explicitly created routed pairs 
fn get_pairs_full_implicit(
    storage: &dyn Storage,
    start_after: Option<[String; 2]>,
    limit: Option<u16>,
) -> Vec<PopulatedPair> {

    let (pair_start_after, pool_start_after, route_start_after) = if start_after.is_some() {
        let denoms = start_after.unwrap();
        let key = key_from(&denoms);

        let pair_exist = PAIRS.has(storage, key.clone());
        let pool_exist = POOLS.has(storage, key.clone());
        let route_exist = ROUTES.has(storage, key.clone());

        if !pair_exist && !pool_exist && !route_exist{
            return vec![]
        }

        let bound = Some(Bound::exclusive(key.clone()));
        if pair_exist {
            (bound, None, None)
        } else if pool_exist {
            (None, bound, None)
        } else {
            (None, None, bound)
        }
    } else {
        (None, None, None)
    };

    let original_limit = limit.unwrap_or(30) as usize;

    let mut limit = original_limit;

    // Get explicitly created pairs first
    let mut pairs = PAIRS
            .range(
                storage,
                pair_start_after,
                None,
                Order::Ascending,
            )
            .take(limit)
            .flat_map(|result| 
                result.map(|(key, pair)| match pair {
                    StoredPairType::Direct { } => find_pool_pair(storage, denoms_from(&key)),
                    StoredPairType::Routed { } => find_route_pair(storage, denoms_from(&key))
                })
            )
            .collect::<StdResult<Vec<PopulatedPair>>>().unwrap();
    
       
    limit = original_limit.checked_sub(pairs.len()).unwrap_or(0);
    
    // Getting direct pools if limit is not reached
    if limit > 0 {
        let pools = POOLS
            .range(
                storage,
                pool_start_after,
                None,
                Order::Ascending,
            )
            .filter_map(|pool_res| {
                if let Ok((key, pool)) = pool_res {
                    // skip explicit
                    if !pair_exists(storage, &denoms_from(&key)) {
                        Some(pool.into())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(limit)
            .collect::<Vec<PopulatedPair>>();

        pairs.extend(pools);
    }

    limit = original_limit.checked_sub(pairs.len()).unwrap_or(0);

    // Getting routed pairs if limit is not reached
    if limit > 0 {
        let pools = ROUTES
            .range(
                storage,
                route_start_after,
                None,
                Order::Ascending,
            )
            .filter_map(|route_res| {
                if let Ok((key, _)) = route_res {
                    let denoms = denoms_from(&key);
                    if !pair_exists(storage, &denoms) {
                        Some(get_routed_pair(storage, denoms, false).unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(limit)
            .collect::<Vec<PopulatedPair>>();

        pairs.extend(pools);
    }

    pairs
}





#[cfg(test)]
mod find_pair_tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn saves_and_finds_pair() {
        let mut deps = mock_dependencies();
        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let saved_pair = find_pair(&deps.storage, pair.denoms()).unwrap();
        assert_eq!(pair, saved_pair);
    }

    #[test]
    fn saves_and_finds_pair_with_denoms_reversed() {
        let mut deps = mock_dependencies();
        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let denoms = [pair.denoms()[1].clone(), pair.denoms()[0].clone()];

        let saved_pair = find_pair(&deps.storage, denoms).unwrap();
        assert_eq!(pair, saved_pair);
    }

    #[test]
    fn find_pair_that_does_not_exist_fails() {
        let deps = mock_dependencies();

        let result = find_pair(&deps.storage, PopulatedPair::default().denoms()).unwrap_err();

        assert_eq!(result, StdError::generic_err("Pair not found"));
    }
}


#[cfg(test)]
mod get_pairs_tests {
    use astrovault::assets::asset::AssetInfo;
    use cosmwasm_std::testing::mock_dependencies;

    use crate::types::pair::PopulatedPair;

    use super::{get_pairs, save_pair};

    #[test]
    fn fetches_all_pairs() {
        let mut deps = mock_dependencies();

        for i in 0..10 {
            let pair = PopulatedPair::from_assets(
                AssetInfo::NativeToken { denom: format!("base_denom_{}", i) },
                AssetInfo::NativeToken { denom: format!("quote_denom_{}", i) }
            );
            save_pair(deps.as_mut().storage, &pair).unwrap();
        }

        let pairs = get_pairs(deps.as_ref().storage, None, None);
        assert_eq!(pairs.len(), 10);
    }

    #[test]
    fn fetches_all_pairs_with_limit() {
        let mut deps = mock_dependencies();
        for i in 0..10 {
            let pair = PopulatedPair::from_assets(
                AssetInfo::NativeToken { denom: format!("base_denom_{}", i) },
                AssetInfo::NativeToken { denom: format!("quote_denom_{}", i) }
            );
            save_pair(deps.as_mut().storage, &pair).unwrap();
        }
        let pairs = get_pairs(deps.as_ref().storage, None, Some(5));
        assert_eq!(pairs.len(), 5);
    }


    #[test]
    fn fetches_all_pairs_with_start_after() {
        let mut deps = mock_dependencies();

        for i in 0..10 {
            let pair = PopulatedPair::from_assets(
                AssetInfo::NativeToken { denom: format!("base_denom_{}", i) },
                AssetInfo::NativeToken { denom: format!("quote_denom_{}", i) }
            );
            save_pair(deps.as_mut().storage, &pair).unwrap();
        }

        let pairs = get_pairs(
            deps.as_ref().storage,
            Some(["base_denom_5".to_string(), "quote_denom_5".to_string()]),
            None,
        );

        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0].base_denom(), "base_denom_6");
    }

    /*


    

    #[test]
    fn fetches_all_pairs_with_start_after_and_limit() {
        let mut deps = mock_dependencies();

        for i in 0..10 {
            let pair = Pair {
                base_denom: format!("base_denom_{}", i),
                quote_denom: format!("quote_denom_{}", i),
                address: Addr::unchecked(format!("address_{}", i)),
                decimal_delta: 0,
                price_precision: 3,
                pool_type: PoolType::Standard
            };

            save_pair(deps.as_mut().storage, &pair).unwrap();
        }

        let pairs = get_pairs(
            deps.as_ref().storage,
            Some(["base_denom_3".to_string(), "quote_denom_3".to_string()]),
            Some(2),
        );

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].base_denom, "base_denom_4");
    }
 */

}
