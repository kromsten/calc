use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::Addr;
use crate::types::pair::{Pair, PoolType};
use super::constants::{DENOM_AARCH, DENOM_UUSDC};


impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_asset: AssetInfo::NativeToken { denom: DENOM_AARCH.to_string() },
            quote_asset: AssetInfo::NativeToken { denom: DENOM_UUSDC.to_string() },
            address: Some(Addr::unchecked("pair-address")),
            pool_type: Some(PoolType::Standard),
            route: None,
        }
    }
}
