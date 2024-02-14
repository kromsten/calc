use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;

/// Represents the type of the astrovault pool.
#[cw_serde]
pub enum PoolType {
    /// Standard pool type.
    Standard,
    /// Stable pool type.
    Stable,
    /// Ratio pool type.
    Ratio,
}

/// Represents a direct unpopulated astrovault pool. Provided on creation or when supplying a custom route.
#[cw_serde]
pub struct Pool {
    /// The address of the pool.
    pub address: String,
    /// The type of pool.
    pub pool_type: PoolType,
    /// Information about the base asset.
    pub base_asset: AssetInfo,
    /// Information about the quote asset.
    pub quote_asset: AssetInfo,
}

/// A populated astrovault pool with local information about indeces of the base and quote assets.
#[cw_serde]
pub struct PopulatedPool {
    /// The address of the pool.
    pub address: String,
    /// The type of pool.
    pub pool_type: PoolType,
    /// Information about the base asset.
    pub base_asset: AssetInfo,
    /// Information about the quote asset.
    pub quote_asset: AssetInfo,
    /// The index of the base asset.
    pub base_index: u32,
    /// The index of the quote asset.
    pub quote_index: u32,
}
