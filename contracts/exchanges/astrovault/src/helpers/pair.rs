use cosmwasm_std::Deps;
use crate::{types::pair::Pair, ContractError};


pub fn pair_exists(
    pair: &Pair,
    _deps: Deps
) -> Result<(), ContractError> {

    if pair.base_asset.equal(&pair.quote_asset) {
        return Err(ContractError::SameAsset {});
    }

    if !(pair.address.is_some() ^ pair.pool_type.is_some()) {
        return Err(ContractError::InvalidPair { 
            msg: String::from("Both address and pool type must be provided for direct pairs") 
        });
    }

    Ok(())
}