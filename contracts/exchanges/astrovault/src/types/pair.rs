use astrovault::assets::asset::{AssetInfo, Asset};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Uint128};
use exchange::msg::Pair as ExchangePair;

use crate::ContractError;

#[cfg(target_arch = "wasm32")]
use crate::helpers::pair::pair_exists;

#[cw_serde]
pub enum PoolType {
    Standard,
    Stable,
    Ratio
}


#[cw_serde]
pub struct Pair {
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub price_precision: u8,
    pub decimal_delta: i8,
    pub address: Option<Addr>,
    pub pool_type: Option<PoolType>,
    pub route: Option<Vec<String>>
}


impl Pair {
    pub fn assets(&self) -> [AssetInfo; 2] {
        [self.base_asset.clone(), self.quote_asset.clone()]
    }
    pub fn denoms(&self) -> [String; 2] {
        [self.base_asset.to_string(), self.quote_asset.to_string()]
    }
    pub fn other_asset(&self, swap_asset: &AssetInfo) -> AssetInfo {
        if self.quote_asset.equal(swap_asset) {
            self.base_asset.clone()
        } else {
            self.quote_asset.clone()
        }
    }

    pub fn base_denom(&self) -> String {
        self.base_asset.to_string()
    }

    pub fn quote_denom(&self) -> String {
        self.quote_asset.to_string()
    }

    pub fn is_pool_pair(&self) -> bool {
        self.address.is_some() && self.pool_type.is_some()
    }

    #[allow(unused_variables)]
    pub fn validate(&self, deps: Deps) -> Result<(), ContractError> {
        if self.base_asset.equal(&self.quote_asset) {
            return Err(ContractError::SameAsset {});
        }

        if self.address.is_some() ^ self.pool_type.is_some() {
            return Err(ContractError::InvalidPair { 
                msg: String::from("Both address and pool type must be provided for direct pairs") 
            });
        };

        if self.is_pool_pair() && (self.route.is_some())  {
            return Err(ContractError::InvalidPair { 
                msg: String::from("Providing route for direct pairs is not supported") 
            });
        };

        if !self.is_pool_pair() && (self.route.is_none())  {
            return Err(ContractError::InvalidPair { 
                msg: String::from("Must provide default route for non-direct pairs") 
            });
        };

        #[cfg(target_arch = "wasm32")]
        pair_exists(self, deps)?;

        Ok(())
    }
}



impl From<Pair> for ExchangePair {
    fn from(val: Pair) -> Self {
        ExchangePair {
            denoms: val.denoms(),
        }
    }
}

#[cw_serde]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}
