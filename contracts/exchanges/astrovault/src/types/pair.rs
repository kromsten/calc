use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;

use super::pool::PoolType;
use super::route::{PopulatedRoute, Route};

/// Represents the type of unpopulated pair provided on creation or when supplying a custom route
#[cw_serde]
pub enum PairType {
    Direct {
        address: String,
        pool_type: PoolType,
    },
    Routed {
        route: Route,
    },
}

/// Represents the type of a populated pair.
/// Serving as a wrapper for populated pools and routes for unifying typing
#[cw_serde]
pub enum PopulatedPairType {
    Direct {
        address: String,
        pool_type: PoolType,
        base_index: u32,
        quote_index: u32,
    },
    Routed {
        route: PopulatedRoute,
    },
}

/// Unpopulated pair supplied for creation or when supplying a custom route.
#[cw_serde]
pub struct Pair {
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub pair_type: PairType,
}

/// Populated pair with local information about indices of the base and quote assets.
//  Serving as a wrapper for populated pools and routes for unifying typing
#[cw_serde]
pub struct PopulatedPair {
    pub base_asset: AssetInfo,
    pub quote_asset: AssetInfo,
    pub pair_type: PopulatedPairType,
}

/// Primarily used for type of explicitly stored pair to simplify the logic
#[cw_serde]
pub enum StoredPairType {
    Direct,
    Routed,
}
