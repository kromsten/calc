use astrovault::assets::asset::AssetInfo;
use crate::types::pair::{Pair, PairType, PoolType};
use super::constants::{DENOM_AARCH, DENOM_UUSDC};


impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_asset: AssetInfo::NativeToken { denom: DENOM_AARCH.to_string() },
            quote_asset: AssetInfo::NativeToken { denom: DENOM_UUSDC.to_string() },
            pair_type: PairType::Direct {
                address: String::from("pair-address"),
                pool_type: PoolType::Standard,
                base_index: None,
                quote_index: None,
            },
        }
    }
}

impl Pair {
    pub fn from_assets(
        base_asset: AssetInfo, 
        quote_asset: AssetInfo
    ) -> Self {
        Pair {
            base_asset,
            quote_asset,
            ..Default::default()
        }
    }
}