use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;


use super::pool::{PoolType, PopulatedPool};



#[cw_serde]
pub struct HopInfo {
    pub address:      String,
    pub pool_type:    PoolType,
    pub asset_info:   AssetInfo,
}



#[cw_serde]
pub struct RouteHop {
    pub denom:     String,
    pub prev:      HopInfo,
    pub next:      Option<HopInfo>,
}




pub type Route              =  Vec<RouteHop>;

pub type PopulatedRoute     =  Vec<PopulatedPool>;

pub type StoredRoute        =  Vec<String>;