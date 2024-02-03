use cosmwasm_std::{ from_json, Binary, Coin, CosmosMsg, Deps, Env, QuerierWrapper};
use astrovault::{
    assets::asset::{Asset, AssetInfo},
    router::state::Hop as AstroHop
};

use exchange::msg::Pair as ExchangePair;

use crate::types::{pair::{Pair, PairType, PopulatedPair, PopulatedPairType}, pool::PopulatedPool};
use crate::types::pool::{Pool, PoolType};
use crate::types::route::{PopulatedRoute, Route};
use crate::ContractError;


use super::{route::route_swap_cosmos_msg, validated::validated_routed_pair};




impl From<Pair> for ExchangePair {
    fn from(val: Pair) -> Self {
        ExchangePair {
            denoms: val.denoms(),
        }
    }
}



impl Pair {

    pub fn base_denom(&self) -> String {
        self.base_asset.to_string()
    }

    pub fn quote_denom(&self) -> String {
        self.quote_asset.to_string()
    }

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

    pub fn pool(&self) -> Pool {
        match self.pair_type.clone() {
            PairType::Direct { 
                address, 
                pool_type,
            
             } => {
                Pool {
                    address,
                    pool_type,
                    base_asset: self.base_asset.clone(),
                    quote_asset: self.quote_asset.clone(),
                }
            },
            _ => panic!("Cannot convert routed pair into direct pool")
        }
    }

    pub fn route(&self) -> Route {
        match &self.pair_type {
            PairType::Routed { route } => {
                route.clone()
            },
            _ => panic!("Cannot convert direct pair into route")
        }
    }


    pub fn is_pool_pair(&self) -> bool {
        match &self.pair_type {
            PairType::Direct { .. } => true,
            _ => false
            
        }
    }

    pub fn is_route_pair(&self) -> bool {
        match &self.pair_type {
            PairType::Routed { .. } => true,
            _ => false
        }
    }


    pub fn route_denoms(&self) -> Vec<String> {
        let route = self.route();
        let mut denoms : Vec<String> = Vec::with_capacity(route.len() + 2);
        denoms.push(self.base_denom());
        for hop in route {
            denoms.push(hop.denom.clone());
        }
        denoms.push(self.quote_denom());

        denoms
    }

    pub fn new_routed(
        base_asset: AssetInfo, 
        quote_asset: AssetInfo,
        route: Route
    ) -> Self {
        Pair {
            base_asset,
            quote_asset,
            pair_type: PairType::Routed { route }
        }
    }

}








impl PopulatedPair {

    pub fn is_pool_pair(&self) -> bool {
        match &self.pair_type {
            PopulatedPairType::Direct { .. } => true,
            _ => false
            
        }
    }

    pub fn is_route_pair(&self) -> bool {
        match &self.pair_type {
            PopulatedPairType::Routed { .. } => true,
            _ => false
        }
    }

    pub fn pool(&self) -> PopulatedPool {
        self.into()
    }

    pub fn route(&self) -> PopulatedRoute {
       self.into()
    }


    pub fn base_denom(&self) -> String {
        self.base_asset.to_string()
    }

    pub fn quote_denom(&self) -> String {
        self.quote_asset.to_string()
    }

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

    pub fn has_asset(&self, asset: &AssetInfo) -> bool {
        self.base_asset.equal(asset) || self.quote_asset.equal(asset)
    }

    pub fn has_denom(&self, denom: &String) -> bool {
        self.base_denom() == *denom || self.quote_denom() == *denom
    }


    pub fn to_astro_hop(
        &self,
        querier:     &QuerierWrapper,
        offer_asset: &AssetInfo,
    ) -> Result<AstroHop, ContractError> {
        self.pool().astro_hop(querier, offer_asset)   
    }


    pub fn swap_msg(
        &self,
        deps:                Deps,
        env:                 Env,
        offer_asset:         Asset,
        target_asset:        Asset,
        route:               Option<Binary>,
        funds:               Vec<Coin>,
    ) -> Result<CosmosMsg, ContractError> {

        let msg = if self.is_pool_pair() {
            let pool = self.pool();
            let swap_msg = pool.swap_msg_cosmos(
                offer_asset.clone(),
                Some(target_asset.amount),
                funds
            )?;
            swap_msg
        } else {

            let routed_pair = if let Some(route) = route {
                let route : Route = from_json(&route)?;
                let pair = Pair::new_routed(
                    self.base_asset.clone(),
                    self.quote_asset.clone(),
                    route
                );
                validated_routed_pair(deps, &pair, Some(offer_asset.info.clone()))?
            } else {
                self.clone()
            };

            let msg = route_swap_cosmos_msg(
                deps,
                env,
                routed_pair,
                offer_asset,
                target_asset,
                funds
            )?;

            msg
        };


        Ok(msg)
 
    }



    pub fn new_direct(
        base_asset: AssetInfo, 
        quote_asset: AssetInfo,
        address: String,
        pool_type: PoolType,
        base_index: u32,
        quote_index: u32,
    ) -> Self {
        PopulatedPair {
            base_asset,
            quote_asset,
            pair_type: PopulatedPairType::Direct {
                pool_type,
                address,
                base_index,
                quote_index,
            },
        }
    }

    pub fn new_routed(
        base_asset:     AssetInfo, 
        quote_asset:    AssetInfo,
        route:          PopulatedRoute
    ) -> Self {
        PopulatedPair {
            base_asset,
            quote_asset,
            pair_type: PopulatedPairType::Routed {
                route,
            },
        }
    }

}


