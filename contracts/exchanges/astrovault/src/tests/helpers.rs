use astrovault::assets::asset::AssetInfo;
use crate::types::{pair::{PopulatedPair, PopulatedPairType}, pool::PoolType};
use super::constants::{DENOM_AARCH, DENOM_UUSDC};


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