use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Item, Map};

use crate::types::pair::StoredPairType;


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


/// Store explicirely created pairs with infomation about pair type
pub const PAIRS             : Map<String, StoredPairType>    = Map::new("pairs_v1");

/// flag that tell whether to use pairs that wasn't explicite created for querying and swapping
const IMPLICT_PAIRS        : Item<bool>           = Item::new("i");


pub fn allow_implicit(storage: &dyn Storage) -> bool {
    IMPLICT_PAIRS.load(storage).unwrap_or(false)
}


pub fn update_allow_implicit(storage: &mut dyn Storage, allow: Option<bool>) -> StdResult<()> {
    IMPLICT_PAIRS.save(storage, &allow.unwrap_or(false))
}
