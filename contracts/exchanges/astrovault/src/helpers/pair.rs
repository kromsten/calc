use astrovault::assets::asset::{Asset, AssetInfo};
use astrovault::router::state::Hop as AstroHop;
use cosmwasm_std::{Binary, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, StdResult};
use crate::helpers::route::route_swap_cosmos_msg;
use crate::types::pair::{PairRoute, PairType, PoolInfo, PoolType};
use crate::types::pair::Pair;
use crate::ContractError;
use exchange::msg::Pair as ExchangePair;

use crate::helpers::balance::to_asset_info;


/* #[cfg(not(test))]
use crate::helpers::pool::query_pool_exist; */



impl Into<PoolInfo> for &Pair {
    fn into(self) -> PoolInfo {
        match self.pair_type.clone() {
            PairType::Direct { address, pool_type, .. } => PoolInfo {
                address: address.clone(),
                pool_type: pool_type.clone(),
                base_asset: self.base_asset.clone(),
                quote_asset: self.quote_asset.clone(),
                base_pool_index: None,
                quote_pool_index: None,
            },
            _ => panic!("Pair is not a direct pool")
        }
    }
}


impl From<PoolInfo> for Pair {
    fn from(info: PoolInfo) -> Pair {
        Pair {
            base_asset: info.base_asset,
            quote_asset: info.quote_asset,
            pair_type: PairType::Direct {
                address: info.address,
                pool_type: info.pool_type,
                base_index: info.base_pool_index,
                quote_index: info.quote_pool_index,
            },
        }
    }
}


impl Into<PairRoute> for &Pair {
    fn into(self) -> PairRoute {
        match self.pair_type.clone() {
            PairType::Routed { route } => route.clone(),
            _ => panic!("Pair is not a routed pair")
        }
    }
}

impl Into<PairRoute> for Pair {
    fn into(self) -> PairRoute {
        match self.pair_type {
            PairType::Routed { route } => route,
            _ => panic!("Pair is not a routed pair")
        }
    }
}


impl From<Pair> for ExchangePair {
    fn from(val: Pair) -> Self {
        ExchangePair {
            denoms: val.denoms(),
        }
    }
}



impl Pair {

    pub fn is_pool_pair(&self) -> bool {
        match &self.pair_type {
            PairType::Direct { .. } => true,
            _ => false
            
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        self.into()
    }

    pub fn route(&self) -> PairRoute {
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

    pub fn route_pools(&self) -> Vec<PoolInfo> {
        let route = self.route();
        let mut pools = Vec::with_capacity(route.len() + 1);
        
        let first = route.first().unwrap().clone();
        pools.push(PoolInfo {
            base_asset: self.base_asset.clone(),
            quote_asset: to_asset_info(first.denom.clone()),
            address: first.prev.address.clone(),
            pool_type: first.prev.pool_type.clone(),
            base_pool_index: None,
            quote_pool_index: None,
        });


        for (index, hop) in route.iter().enumerate().skip(1) {
            let prev_hop = route.get(index - 1).unwrap();

            pools.push(PoolInfo {
                base_asset: to_asset_info(prev_hop.denom.clone()),
                quote_asset: to_asset_info(hop.denom.clone()),
                address: hop.prev.address.clone(),
                pool_type: hop.prev.pool_type.clone(),
                base_pool_index: None,
                quote_pool_index: None,
            });
        }

        let last = route.last().unwrap().clone();

        pools.push(PoolInfo {
            base_asset: to_asset_info(last.denom.clone()),
            quote_asset: self.quote_asset.clone(),
            address: last.next.address.clone(),
            pool_type: last.next.pool_type.clone(),
            base_pool_index: None,
            quote_pool_index: None,
        });

        pools
    }


    pub fn route_pairs(&self) -> Vec<Pair> {
        let route = self.route();
        let mut pairs = Vec::with_capacity(route.len() + 1);
        
        let first = route.first().unwrap().clone();
        pairs.push(Pair::new_direct(
            self.base_asset.clone(), 
            to_asset_info(first.denom.clone()), 
            first.prev.address.clone(), 
            first.prev.pool_type.clone(), 
            None, 
            None
        ));

        for (index, hop) in route.iter().enumerate().skip(1) {
            let prev_hop = route.get(index - 1).unwrap();
            pairs.push(Pair::new_direct(
                to_asset_info(prev_hop.denom.clone()),
                to_asset_info(hop.denom.clone()),
                hop.prev.address.clone(),
                hop.prev.pool_type.clone(),
                None,
                None
            ));
        }

        let last = route.last().unwrap().clone();
        pairs.push(Pair::new_direct(
            to_asset_info(last.denom.clone()),
            self.quote_asset.clone(),
            last.next.address.clone(),
            last.next.pool_type.clone(),
            None,
            None
        ));
    
        pairs
    }


    pub fn route_denoms(&self) -> Vec<String> {
        let route = self.route();
        let mut denoms = Vec::with_capacity(route.len() + 1);
        denoms.push(self.base_denom());
        for hop in route.iter() {
            denoms.push(hop.denom.clone());
        }
        denoms.push(self.quote_denom());
        denoms
    }

    pub fn valid_route_hops(&self) -> StdResult<()> {
        let denoms = self.route_denoms();

        let mut denoms_set = denoms.clone();
        denoms_set.sort();
        denoms_set.dedup();

        if denoms_set.len() != denoms.len() {
            return Err(StdError::generic_err("Route denoms are not unique"));
        }
        Ok(())
    }


    pub fn to_astro_hop(
        &self,
        querier:     &QuerierWrapper,
        offer_asset: &AssetInfo,
    ) -> StdResult<AstroHop> {
        self.pool_info().to_astro_hop(querier, offer_asset)   
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
            let pool = self.pool_info();
            let swap_msg = pool.pool_swap_cosmos_msg(
                offer_asset,
                Some(target_asset.amount),
                funds
            )?;
            swap_msg
        } else {
            let msg = route_swap_cosmos_msg(
                deps,
                env,
                &self,
                offer_asset,
                target_asset,
                route,
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
        base_index: Option<u32>,
        quote_index: Option<u32>,
    ) -> Self {
        Pair {
            base_asset,
            quote_asset,
            pair_type: PairType::Direct {
                pool_type,
                address,
                base_index,
                quote_index,
            },
        }
    }

    pub fn new_routed(
        base_asset: AssetInfo, 
        quote_asset: AssetInfo,
        route: PairRoute
    ) -> Self {
        Pair {
            base_asset,
            quote_asset,
            pair_type: PairType::Routed {
                route,
            },
        }
    }

}



/* 


pub fn pair_creatable(
    deps: Deps,
    pair: &Pair,
) -> Result<(), ContractError> {
    ensure!(!pair_is_stored(deps.storage, pair), ContractError::PairExist {});

    if pair.is_pool_pair() {
        #[cfg(not(test))]
        query_pool_exist(deps, &pair)?;
    } else {

    }


    Ok(())
}
*/