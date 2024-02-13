use cosmwasm_std::{Order, StdError, StdResult, Storage};
use cw_storage_plus::Map;

use crate::{helpers::route::populated_route_denoms, state::{common::sorted_denoms, pools::get_pool}, types::{pair::PopulatedPair, pool::PopulatedPool, route::{PopulatedRoute, StoredRoute}}, ContractError}
;

use super::{common::{allow_implicit, key_from, PAIRS}, pools::save_pool};



/// list if intermidiate denoms between base and quote assets
/// sorted (base.denom, quote.denom) -> List of denoms where
/// each two consecutive denoms are pools that exist in storage
pub const ROUTES          : Map<String, StoredRoute>   = Map::new("sr_v1");


pub fn route_exists(
    storage:    &dyn Storage, 
    denoms:     &[String; 2],
) -> bool {
    ROUTES.has(storage, key_from(denoms))
}



pub fn get_stored_route(
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
    let key = key_from(&denoms);

    let [base, quote] = if reverse {
        [denoms[1].clone(), denoms[0].clone()]
    } else {
        denoms
    };

    
    let mut route = get_stored_route(storage, key.clone(), reverse)?;
    let mut hop_pools : Vec<PopulatedPool> = Vec::with_capacity(route.len() + 2);
    

    let first = route.first().unwrap().clone();
    let mut last = route.last().unwrap().clone();


    let fist_pool = match get_pool(storage, key_from(&[base.clone(), first.clone()])) {
        Ok(pool) => pool,
        Err(_) => {
            let key = key_from(&[base.clone(), last.clone()]);
            last = first.clone();
            route.reverse();
            get_pool(storage, key)?
        }
        
    };
    

    hop_pools.push(fist_pool);

    for (index, denom) in route.iter().enumerate().skip(1) {
        let prev = route.get(index - 1).unwrap().clone();

        hop_pools.push(
            get_pool(storage, key_from(&[denom.clone(), prev]))?
        );

    }


    hop_pools.push(
        get_pool(storage, key_from(&[last, quote]))?
    );
    println!("got last");
    
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

    // save info that pair exists
    PAIRS.save(storage, key.clone(), &pair.into())?;

    let route = pair.route();

    // save all intermediary pool infos
    route
    .iter()
    .map(|pool| save_pool(storage, key_from(&pool.denoms()), pool))
    .collect::<Result<Vec<()>, ContractError>>()?;

    // get all unique denoms in route
    let denoms = populated_route_denoms(&route);

    // iterate over each denom
    for (base_index, base) in denoms.iter().enumerate() {

        // skip one following since it's a direct pool and get iterate over the rest
        for (quote_index, quote) in denoms.iter().enumerate().skip(base_index + 2) {
            // get all hops denoms between base and quote and store a route
            let between = &denoms[base_index + 1..quote_index].to_vec();
            ROUTES.save(storage, key_from(&[base.clone(), quote.clone()]), between)?;
        }
    }
    Ok(())
}




pub fn find_route(
    storage:    &dyn Storage, 
    denoms:     [String; 2],
    reverse:    bool
) -> StdResult<PopulatedRoute> {

    let pair = PAIRS.load(storage, key_from(&denoms));
    let routed = get_route(storage, denoms, reverse);

    if pair.is_ok() || allow_implicit(storage) {
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




fn get_routes_with_pool_hop(
    storage:    &dyn Storage, 
    key:        String,
) -> Vec<String> {

    ROUTES
        .range(storage, None, None, Order::Ascending)
        .filter_map(|item_res| {
            if let Ok((k, route)) = item_res {
                // iterate over route and check if any two consecutive denoms turned are equal to the key
                for (index, denom) in route.iter().enumerate().skip(1) {
                    if key == key_from(&[route[index - 1].clone(), denom.clone()]) {
                        return Some(k);
                    }
                }
                None
                
            } else {
                None
            }
        })
        .collect()
}



pub fn delete_routes_with_pool(
    storage:    &mut dyn Storage, 
    key:        String,
) {
    for route in get_routes_with_pool_hop(storage, key) {
        ROUTES.remove(storage, route);
    }
}



#[cfg(test)]
mod saving_routed_pairs_tests {
    use astrovault::assets::asset::AssetInfo;
    use cosmwasm_std::{testing::mock_dependencies, Order, Storage};

    use crate::{state::{common::{key_from, update_allow_implicit, PAIRS}, pairs::get_pairs, pools::POOLS, routes::save_routed_pair}, types::{pair::PopulatedPair, pool::PopulatedPool}};

    use super::route_exists;


    fn default_denoms() -> [String; 2] {
        [String::from("A"), String::from("F")]
    }

    fn default_key() -> String {
        key_from(&default_denoms())
    }

    fn default_routed_pair() -> PopulatedPair {
        PopulatedPair::from_assets_routed(
            AssetInfo::NativeToken { denom: format!("A") },
            AssetInfo::NativeToken { denom: format!("F") },
            vec![
                PopulatedPool::from_assets(
                    AssetInfo::NativeToken { denom: format!("A") },
                    AssetInfo::NativeToken { denom: format!("B") }
                ),
                PopulatedPool::from_assets(
                    AssetInfo::NativeToken { denom: format!("B") },
                    AssetInfo::NativeToken { denom: format!("C") }
                ),
                PopulatedPool::from_assets(
                    AssetInfo::NativeToken { denom: format!("C") },
                    AssetInfo::NativeToken { denom: format!("D") }
                ),
                PopulatedPool::from_assets(
                    AssetInfo::NativeToken { denom: format!("D") },
                    AssetInfo::NativeToken { denom: format!("E") }
                ),
                PopulatedPool::from_assets(
                    AssetInfo::NativeToken { denom: format!("E") },
                    AssetInfo::NativeToken { denom: format!("F") }
                ),
            ]
        )
    }

    fn pairs_keys_len(storage: &dyn Storage) -> usize {
        PAIRS.keys(storage, None, None, Order::Ascending).count()
    }

 
    #[test]
    fn all_pair_pools_and_route_saved() {
        let mut deps = mock_dependencies();
        let deps = deps.as_mut();

        let pair = default_routed_pair();
        save_routed_pair(deps.storage, &pair).unwrap();

        assert_eq!(pairs_keys_len(deps.storage), 1);
        assert!(PAIRS.has(deps.storage, default_key()));
        assert!(route_exists(deps.storage, &default_denoms()));

        for pool in pair.route() {
            assert!(POOLS.has(deps.storage, key_from(&pool.denoms())))
        }

        let pairs = get_pairs(deps.storage, None, None);

        assert_eq!(pairs.len(), 1);
    }


    #[test]
    fn implicit_routed_pairs_exist() {
        let mut deps = mock_dependencies();
        let deps = deps.as_mut();
        update_allow_implicit(deps.storage, Some(true)).unwrap();


        let pair = default_routed_pair();
        let route = pair.route();
        save_routed_pair(deps.storage, &pair).unwrap();

        assert_eq!(pairs_keys_len(deps.storage), 1);
        assert!(route_exists(deps.storage, &default_denoms()));

        for pool in route.iter() {
            assert!(POOLS.has(deps.storage, key_from(&pool.denoms())))
        }

        let pairs = get_pairs(deps.storage, None, None);

        println!("\n\n");
        for pair in pairs.iter() {
            println!("{:?}\n", pair);
        }
        println!("\n\n");

        
        let direct_pool_count = route.len();

        // at least 0 by definition
        let len = route.len();

        let routed_count = route
            .iter()
            .enumerate()
            .take(len - 1)
            .fold(0usize, |acc, (index, _)| {
                acc + (route.len() - 1 - index)
            });
        
        /*
            A -> B   :  1
            A -> C   :  2
            A -> D   :  3
            A -> E   :  4
            A -> F   :  5
            
            B -> C   :  6
            B -> D   :  7
            B -> E   :  8
            B -> F   :  9

            C -> D   : 10
            C -> E   : 11
            C -> F   : 12

            D -> E   : 13
            D -> F   : 14

            E -> F   : 15
         */


        let total =  direct_pool_count + routed_count;

        assert_eq!(pairs.len(), total);
        assert_eq!(total, 15);
    }



/* 
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
    } */



}
