use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Map;
use crate::{types::{pair::PopulatedPair, pool::PopulatedPool}, ContractError};

use super::common::{allow_implicit, key_from, PAIRS};



/// pool info to populate both direct pool pairs and routed pairs
/// sorted (base.denom, quote.denom) -> PopulatedPool
pub const POOLS             : Map<String, PopulatedPool> = Map::new("p_v1");


pub fn pool_exists(
    storage: &dyn Storage, 
    denoms: &[String; 2]
) -> bool {
    POOLS.has(storage, key_from(denoms))
}


pub fn get_pool(
    storage: &dyn Storage, 
    key: String
) -> StdResult<PopulatedPool> {
    POOLS.load(storage, key)
}

pub fn get_pool_pair(
    storage: &dyn Storage, 
    key: String
) -> StdResult<PopulatedPair> {
    Ok(POOLS.load(storage, key)?.into())
}


pub fn save_pool(
    storage: &mut dyn Storage,
    key:     String,
    pool:    &PopulatedPool
) -> Result<(), ContractError> {
    POOLS.save(storage, key, pool)?;
    Ok(())
}

pub fn save_pool_pair(
    storage: &mut dyn Storage,
    pair:    &PopulatedPair
) -> Result<(), ContractError> {
    let key = key_from(&pair.denoms());
    PAIRS.save(storage, key.clone(), &pair.into())?;
    save_pool(storage, key, &pair.pool())
}



pub fn find_pool(storage: &dyn Storage, denoms: [String; 2]) -> StdResult<PopulatedPool> {

    let key = key_from(&denoms);
    let pair = PAIRS.load(storage, key.clone());
    
    if pair.is_ok() {
        return get_pool(storage, key);
    }

    let pool = get_pool(storage, key);

    if allow_implicit(storage) {
        pool
        
    } else {
        Err(StdError::generic_err(
            if pool.is_ok() {
                "Direct pool pair exist but implicit pairs are not allowed"
            } else {
                "Direct pool pair is not found"
            }
        ))
    }
}

