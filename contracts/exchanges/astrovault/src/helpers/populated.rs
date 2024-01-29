use cosmwasm_std::{ensure, QuerierWrapper};

use crate::types::pair::{PopulatedPair, PopulatedPairType};
use crate::types::pool::{Pool, PopulatedPool};
use crate::types::route::PopulatedRoute;
use crate::ContractError;

use super::pool::query_assets;



impl Into<PopulatedPool> for &PopulatedPair {
    fn into(self) -> PopulatedPool {
        match self.pair_type.clone() {
            PopulatedPairType::Direct { 
                address, 
                pool_type, 
                base_index, 
                quote_index 
            } => {
                PopulatedPool {
                    address,
                    pool_type,
                    base_index,
                    quote_index,
                    base_asset: self.base_asset.clone(),
                    quote_asset: self.quote_asset.clone(),
                }
            },
            _ => panic!("Cannot convert route pair into pool")
        }
    }
}


impl From<PopulatedPool> for PopulatedPair {
    fn from(pool: PopulatedPool) -> Self {
        PopulatedPair {
            base_asset: pool.base_asset,
            quote_asset: pool.quote_asset,
            pair_type: PopulatedPairType::Direct {
                address: pool.address,
                pool_type: pool.pool_type,
                base_index: pool.base_index,
                quote_index: pool.quote_index,
            },
        }
    }
}



impl Into<PopulatedRoute> for &PopulatedPair {
    fn into(self) -> PopulatedRoute {
        match &self.pair_type {
            PopulatedPairType::Routed { route } => {
                route.clone()
            },
            _ => panic!("Cannot convert route pair into route")
        }
    }
}


impl From<PopulatedRoute> for PopulatedPair {
    fn from(route: PopulatedRoute) -> Self {
        PopulatedPair {
            base_asset: route.first().unwrap().base_asset.clone(),
            quote_asset: route.last().unwrap().quote_asset.clone(),
            pair_type: PopulatedPairType::Routed {
                route
            },
        }
    }
}




pub fn populated_pool(
    querier:    &QuerierWrapper,
    pool:       &Pool,
) -> Result<PopulatedPool, ContractError> {

    let assets = query_assets(
        querier, 
        &pool.address, 
        &pool.pool_type
    )?;

    let base_pos = assets.iter().position(|a| a.info.equal(&pool.base_asset));
    let quot_pos = assets.iter().position(|a| a.info.equal(&pool.quote_asset));

    ensure!(base_pos.is_some(),   ContractError::AssetQueryFail{});
    ensure!(quot_pos.is_some(),   ContractError::AssetQueryFail{});
    ensure!(base_pos != quot_pos, ContractError::AssetQueryFail{});


    Ok(PopulatedPool {
        address: pool.address.clone(),
        pool_type: pool.pool_type.clone(),
        base_asset: pool.base_asset.clone(),
        quote_asset: pool.quote_asset.clone(),
        base_index: base_pos.unwrap() as u32, 
        quote_index: quot_pos.unwrap() as u32,
    })

}
