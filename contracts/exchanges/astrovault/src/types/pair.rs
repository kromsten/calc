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
    /// Direct pool info connecting the hop asset to the previous asset in route
    pub prev:      HopSide,
    /// Direct pool info connecting the hop asset to the next asset in route.
    /// Must be specified for the last hop in the route.
    pub next:      Option<HopSide>,
}


#[cw_serde]
pub struct PoolInfo {
    pub address:       String,
    pub pool_type:     PoolType,
    pub base_asset:    AssetInfo,
    pub quote_asset:   AssetInfo,
    pub base_index:    Option<u32>,
    pub quote_index:   Option<u32>,
}


pub type PairRoute = Vec<HopInfo>;


#[cw_serde]
pub enum PairType {
    Direct {
        address:         String,
        pool_type:       PoolType,
        // to be auto-populated inside contract
        base_index:      Option<u32>,
        // no need to sepcify on the client side
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