use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use super::pool::{PoolType, PopulatedPool};

/// Represents information about side of previous or the next side of the hop.
#[cw_serde]
pub struct HopInfo {
    /// astrovault pool address.
    pub address: String,
    /// The type of pool.
    pub pool_type: PoolType,
    /// Information about the asset.
    pub asset_info: AssetInfo,
}

/// Represents a hop within a route provided by an admin or a user wanting a custom route
#[cw_serde]
pub struct RouteHop {
    /// The denomination of the asset. Both native token or contract address.
    pub denom: String,
    /// Information about the previous hop.
    pub prev: HopInfo,
    /// Information about the next hop. Only needed for the last hop in the route. Ignored otherwise
    pub next: Option<HopInfo>,
}

/// Represents a route, which is a collection of route hops
pub type Route = Vec<RouteHop>;
/// Represents a populated route, which is a collection of populated pools including pools with the base and quote assets
pub type PopulatedRoute = Vec<PopulatedPool>;
/// Represents a stored route. Doesn't include base and quote assets
pub type StoredRoute = Vec<String>;