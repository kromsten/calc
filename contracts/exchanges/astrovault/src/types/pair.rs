use astrovault::assets::asset::{AssetInfo, Asset};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Uint128};
use exchange::msg::Pair as ExchangePair;

use crate::helpers::balance::to_asset_info;
use crate::ContractError;
use crate::helpers::pair::pair_creatable;

#[cw_serde]
pub enum PoolType {
    Standard,
    Stable,
    Ratio
}

#[cw_serde]
pub struct PairHop {
    /// refers to the pool type between the current asset and the previous asset in the route
    pub pool_type: PoolType,
    /// pair contract address. same as the pool type
    pub address:   Addr,

    pub denom:     String
}


#[cw_serde]
pub struct Pair {
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub address: Option<Addr>,
    pub pool_type: Option<PoolType>,
    pub route: Option<Vec<PairHop>>
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

    pub fn route_pools(&self) -> Vec<Pair> {
        let route = self.route.as_ref().unwrap().clone();
        let mut pools_pairs = Vec::with_capacity(route.len() + 1);

        let first = route.first().unwrap();
        pools_pairs.push(Pair {
            base_asset: self.base_asset.clone(),
            quote_asset: to_asset_info(first.denom.clone()),
            address: Some(first.address.clone()),
            pool_type: Some(first.pool_type.clone()),
            route: None,
        });

        for (index, hop) in route.iter().enumerate().skip(1) {
            let prev_hop = route.get(index - 1).unwrap();
            pools_pairs.push(Pair {
                base_asset: to_asset_info(prev_hop.denom.clone()),
                quote_asset: to_asset_info(hop.denom.clone()),
                address: Some(hop.address.clone()),
                pool_type: Some(hop.pool_type.clone()),
                route: None,
            });
        }

        let last = route.last().unwrap();
        pools_pairs.push(Pair {
            base_asset: to_asset_info(last.denom.clone()),
            quote_asset: self.quote_asset.clone(),
            address: Some(last.address.clone()),
            pool_type: Some(last.pool_type.clone()),
            route: None,
        });
    
        pools_pairs
    }
    

    pub fn route_assets(&self) -> Vec<[AssetInfo; 2]> {
        let route = self.route.as_ref().unwrap().clone();
        let mut assets = Vec::with_capacity(route.len() + 1);

        let first = to_asset_info(route.first().unwrap().denom.clone());
        assets.push([self.base_asset.clone(), first.clone()]);

        for (index, hop) in route.iter().enumerate().skip(1) {
            let prev_hop = route.get(index - 1).unwrap();
            assets.push([to_asset_info(prev_hop.denom.clone()), to_asset_info(hop.denom.clone())]);
        }

        let last = to_asset_info(route.last().unwrap().denom.clone());
        assets.push([last.clone(), self.quote_asset.clone()]);

        assets
    }


    pub fn route_denoms(&self) -> Vec<[String; 2]> {
        let route = self.route.as_ref().unwrap().clone();
        let mut denoms = Vec::with_capacity(route.len() + 1);

        let first = route.first().unwrap();
        denoms.push([self.base_asset.to_string(), first.denom.clone()]);

        for (index, hop) in route.iter().enumerate().skip(1) {
            let prev_hop = route.get(index - 1).unwrap();
            denoms.push([prev_hop.denom.clone(), hop.denom.clone()]);
        }

        let last = route.last().unwrap();
        denoms.push([last.denom.clone(), self.quote_asset.to_string()]);

        denoms
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

        if self.route.is_some() {
            let route = self.route.as_ref().unwrap();
            if route.len() < 1 {
                return Err(ContractError::InvalidPair { 
                    msg: String::from("Route must have at least one hop asset") 
                });
            }
        }

        pair_creatable(deps, self)?;

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
