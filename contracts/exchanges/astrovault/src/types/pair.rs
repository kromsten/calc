use astrovault::assets::asset::{AssetInfo, Asset};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use exchange::msg::Pair as ExchangePair;

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
    pub address: Addr,
    pub decimal_delta: i8,
    pub price_precision: u8,
    pub pool_type: PoolType,
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
