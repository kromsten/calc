use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;

use super::pool::PoolType;
use super::route::{Route, PopulatedRoute};


/// Represents the type of unnpopulated pair provided on creation or when supplying a custom route
#[cw_serde]
pub enum PairType {
    /// Direct pair type.
    Direct {
        /// The address of the pool.
        address: String,
        /// The type of pool.
        pool_type: PoolType,
    },
    /// Routed pair type.
    Routed {
        /// The route for the pair.
        route: Route,
    },
}

/// Represents the type of a populated pair. 
/// Serving as a wrapper for populated pools and routes for unifying typing 
#[cw_serde]
pub enum PopulatedPairType {
    /// Direct populated pair type.
    Direct {
        /// The address of the pool.
        address: String,
        /// The type of pool.
        pool_type: PoolType,
        /// The index of the base asset.
        base_index: u32,
        /// The index of the quote asset.
        quote_index: u32,
    },
    /// Routed populated pair type.
    Routed {
        /// The populated route for the pair including pools with the base and quote assets.
        route: PopulatedRoute,
    },
}

/// Unpopulated pair supplied for creation or when supplying a custom route.
#[cw_serde]
pub struct Pair {
    /// Information about the base asset.
    pub base_asset: AssetInfo,
    /// Information about the quote asset.
    pub quote_asset: AssetInfo,
    /// The type of pair.
    pub pair_type: PairType,
}

/// Populated pair with local information about indeces of the base and quote assets.
//  Serving as a wrapper for populated pools and routes for unifying typing
#[cw_serde]
pub struct PopulatedPair {
    /// Information about the base asset.
    pub base_asset: AssetInfo,
    /// Information about the quote asset.
    pub quote_asset: AssetInfo,
    /// The type of populated pair.
    pub pair_type: PopulatedPairType,
}

/// Primarly used for type of explicitely stored pair to simplify the logic 
#[cw_serde]
pub enum StoredPairType {
    /// Direct stored pair type.
    Direct,
    /// Routed stored pair type.
    Routed,
}