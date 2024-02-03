use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;

use super::pool::PoolType;
use super::route::{Route, PopulatedRoute};


#[cw_serde]
pub enum PairType {
    Direct {
        address:   String,
        pool_type: PoolType,
    },
    Routed {
        route: Route
    },
}


#[cw_serde]
pub enum PopulatedPairType {
    Direct {
        address:         String,
        pool_type:       PoolType,
        base_index:      u32,
        quote_index:     u32,
    },
    Routed {
        route: PopulatedRoute
    },
}




#[cw_serde]
pub struct Pair {
    pub base_asset:  AssetInfo,
    pub quote_asset: AssetInfo,
    pub pair_type:   PairType,
}



#[cw_serde]
pub struct PopulatedPair {
    pub base_asset:  AssetInfo,
    pub quote_asset: AssetInfo,
    pub pair_type:   PopulatedPairType,
}


#[cw_serde]
pub enum StoredPairType {
    Direct,
    Routed
}