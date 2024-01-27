use crate::types::pair::Pair;
use cosmwasm_std::{Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

const PAIRS         : Map<String, Pair> = Map::new("pairs_v1");
const ROUTE_PAIRS   : Map<String, Pair> = Map::new("rpairs_v1");


fn key_from(mut denoms: [String; 2]) -> String {
    denoms.sort();
    format!("{}-{}", denoms[0], denoms[1])
}


pub fn save_route_pair(storage: &mut dyn Storage, pair: &Pair) -> StdResult<()> {
    ROUTE_PAIRS.save(storage, key_from(pair.denoms()), &pair)
}


pub fn find_route_pair(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<Pair> {
    let key = key_from(denoms);
    if let Ok(pair) =  ROUTE_PAIRS.load(storage, key.clone()) {
        return Ok(pair);
    } else if let Ok(pair) = PAIRS.load(storage, key) {
        return Ok(pair);
    } else {
        return Err(StdError::generic_err("Pair not found"));
    }
}

pub fn route_pair_exists(storage: &dyn Storage, denoms: [String; 2]) -> bool {
    let key = key_from(denoms);
    if ROUTE_PAIRS.has(storage, key.clone()) {
        true
    } else {
        PAIRS.has(storage, key)
    }
}


pub fn pair_exists(storage: &dyn Storage, pair: &Pair) -> bool {
    PAIRS.has(storage, key_from(pair.denoms()))
}

pub fn save_pair(storage: &mut dyn Storage, pair: &Pair) -> StdResult<()> {
    let key = key_from(pair.denoms());
    if ROUTE_PAIRS.has(storage, key.clone()) {
        ROUTE_PAIRS.remove(storage, key.clone());
    }
    PAIRS.save(storage, key_from(pair.denoms()), &pair)
}

pub fn find_pair(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<Pair> {
    PAIRS.load(storage, key_from(denoms))
}

pub fn delete_pair(storage: &mut dyn Storage, pair: &Pair) {
    PAIRS.remove(storage, key_from(pair.denoms()))
}

pub fn get_pairs(
    storage: &dyn Storage,
    start_after: Option<[String; 2]>,
    limit: Option<u16>,
) -> Vec<Pair> {
    PAIRS
        .range(
            storage,
            start_after.map(|denoms| Bound::exclusive(key_from(denoms))),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(30) as usize)
        .flat_map(|result| result.map(|(_, pair)| pair))
        .collect::<Vec<Pair>>()
}


#[cfg(test)]
mod find_pair_tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn saves_and_finds_pair() {
        let mut deps = mock_dependencies();
        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let saved_pair = find_pair(&deps.storage, pair.denoms()).unwrap();
        assert_eq!(pair, saved_pair);
    }

    #[test]
    fn saves_and_finds_pair_with_denoms_reversed() {
        let mut deps = mock_dependencies();
        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let denoms = [pair.denoms()[1].clone(), pair.denoms()[0].clone()];

        let saved_pair = find_pair(&deps.storage, denoms).unwrap();
        assert_eq!(pair, saved_pair);
    }

    #[test]
    fn find_pair_that_does_not_exist_fails() {
        let deps = mock_dependencies();

        let result = find_pair(&deps.storage, Pair::default().denoms()).unwrap_err();

        assert!(result.to_string().starts_with("type: astrovault_calc::types::pair"));
    }
}


#[cfg(test)]
mod get_pairs_tests {
    use astrovault::assets::asset::AssetInfo;
    use cosmwasm_std::testing::mock_dependencies;

    use crate::types::pair::Pair;

    use super::{get_pairs, save_pair};

   #[test]
    fn fetches_all_pairs() {
        let mut deps = mock_dependencies();

        for i in 0..10 {
            let pair = Pair::from_assets(
                AssetInfo::NativeToken { denom: format!("base_denom_{}", i) },
                AssetInfo::NativeToken { denom: format!("quote_denom_{}", i) }
            );
            save_pair(deps.as_mut().storage, &pair).unwrap();
        }

        let pairs = get_pairs(deps.as_ref().storage, None, None);
        assert_eq!(pairs.len(), 10);
    }

    /*

    #[test]
    fn fetches_all_pairs_with_limit() {
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

        let pairs = get_pairs(deps.as_ref().storage, None, Some(5));

        assert_eq!(pairs.len(), 5);
    }

    #[test]
    fn fetches_all_pairs_with_start_after() {
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
            Some(["base_denom_5".to_string(), "quote_denom_5".to_string()]),
            None,
        );

        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0].base_denom, "base_denom_6");
    }

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
