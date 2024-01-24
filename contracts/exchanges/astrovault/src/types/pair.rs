use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub enum PoolType {
    Standard,
    Stable,
    Ratio
}


#[cw_serde]
pub struct HopSide {
    pub address:   String,
    pub pool_type: PoolType,
}

#[cw_serde]
pub struct HopInfo {
    pub denom:     String,
    pub prev:      HopSide,
    pub next:      HopSide,
}


#[cw_serde]
pub struct PoolInfo {
    pub address:            String,
    pub pool_type:          PoolType,
    pub base_asset:         AssetInfo,
    pub quote_asset:        AssetInfo,
    pub base_pool_index:    Option<u32>,
    pub quote_pool_index:   Option<u32>,
}

pub type PairRoute = Vec<HopInfo>;

#[cw_serde]
pub enum PairType {
    Direct {
        address:         String,
        pool_type:       PoolType,
        base_index:      Option<u32>,
        quote_index:     Option<u32>,
    },
    Routed {
        route: PairRoute
    },
}


#[cw_serde]
pub struct Pair {
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub pair_type: PairType,
}