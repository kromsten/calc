use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;

/// Represents the type of the astrovault pool.
#[cw_serde]
pub enum PoolType {
    Standard,
    Stable,
    Ratio,
}

/// Represents a direct unpopulated astrovault pool. Provided on creation or when supplying a custom route.
#[cw_serde]
pub struct Pool {
    pub address: String,
    pub pool_type: PoolType,
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
}

/// A populated astrovault pool with local information about indices of the base and quote assets.
#[cw_serde]
pub struct PopulatedPool {
    pub address: String,
    pub pool_type: PoolType,
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub base_index: u32,
    pub quote_index: u32,
}
