use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::Addr;

use crate::types::pair::{Pair, PoolType};

use super::constants::{DENOM_AARCH, DENOM_UUSDC};

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_asset: AssetInfo::NativeToken { denom: DENOM_AARCH.to_string() },
            quote_asset: AssetInfo::NativeToken { denom: DENOM_UUSDC.to_string() },
            address: Addr::unchecked("pair-address"),
            decimal_delta: 0,
            price_precision: 3,
            pool_type: PoolType::Standard
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
}
