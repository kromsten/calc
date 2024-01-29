use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Item, Map};

use crate::types::pair::StoredPair;


pub fn sorted_denoms(denoms: &[String; 2]) -> [String; 2] {
    let mut denoms = denoms.clone();
    denoms.sort();
    denoms
}


pub fn key_from(denoms: &[String; 2]) -> String {
    let [base, quote] = sorted_denoms(denoms);
    format!("{}-{}", base, quote)
}

pub fn denoms_from(key: &str) -> [String; 2] {
    let mut denoms = key.split('-');
    [denoms.next().unwrap().to_string(), denoms.next().unwrap().to_string()]
}


/// explicitly added pairs with minimal info and exted fields
/// sorted (base.denom, quote.denom) -> Direct / Routed info
pub const PAIRS             : Map<String, StoredPair>    = Map::new("pairs_v1");


const IMPLICT_PAIRS        : Item<bool>           = Item::new("i");


pub fn allow_implicit(storage: &dyn Storage) -> bool {
    IMPLICT_PAIRS.load(storage).unwrap()
}


pub fn update_allow_implicit(storage: &mut dyn Storage, allow: Option<bool>) -> StdResult<()> {
    IMPLICT_PAIRS.save(storage, &allow.unwrap_or(false))
}
