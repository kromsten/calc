use astrovault::assets::asset::AssetInfo;
use crate::types::{pair::{Pair, PopulatedPair, PopulatedPairType}, pool::{Pool, PoolType, PopulatedPool}};
use super::constants::{DENOM_AARCH, DENOM_UOSMO, DENOM_UUSDC};


impl Default for PopulatedPair {
    fn default() -> Self {
        PopulatedPair {
            base_asset: AssetInfo::NativeToken { denom: DENOM_AARCH.to_string() },
            quote_asset: AssetInfo::NativeToken { denom: DENOM_UUSDC.to_string() },
            pair_type: PopulatedPairType::Direct {
                address: String::from("pair-address"),
                pool_type: PoolType::Standard,
                base_index: 0,
                quote_index: 1,
            },
        }
    }
}

impl PopulatedPair {
    pub fn from_assets(
        base_asset: AssetInfo, 
        quote_asset: AssetInfo
    ) -> Self {
        PopulatedPair {
            base_asset,
            quote_asset,
            ..Default::default()
        }
    }
}

impl PopulatedPool {

    pub fn from_pool(pool: &Pool) -> Self {

        let (base_index, quote_index) = 
            if pool.base_asset.to_string() == DENOM_UOSMO ||
                pool.quote_asset.to_string() == DENOM_UUSDC
        {
            (0, 1)
        } else {
            (1, 0)
        };

        PopulatedPool {
            address: pool.address.clone(),
            pool_type: pool.pool_type.clone(),
            base_asset: pool.base_asset.clone(),
            quote_asset: pool.quote_asset.clone(),
            base_index,
            quote_index,
        }
    }

    pub fn from_pair(pair: &Pair) -> Self {
        let pool = pair.pool();
        PopulatedPool::from_pool(&pool)
    }

    pub fn reversed(&self) -> Self {
        PopulatedPool {
            address: self.address.clone(),
            pool_type: self.pool_type.clone(),
            base_asset: self.quote_asset.clone(),
            quote_asset: self.base_asset.clone(),
            base_index: self.quote_index,
            quote_index: self.base_index,
        }
    }

}