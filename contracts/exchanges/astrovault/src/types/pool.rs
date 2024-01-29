use astrovault::assets::asset::AssetInfo;
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub enum PoolType {
    Standard,
    Stable,
    Ratio
}


#[cw_serde]
pub struct Pool {
    pub address:      String,
    pub pool_type:    PoolType,
    pub base_asset:   AssetInfo,
    pub quote_asset:  AssetInfo,
}


#[cw_serde]
pub struct PopulatedPool {
    pub address:       String,
    pub pool_type:     PoolType,
    pub base_asset:    AssetInfo,
    pub quote_asset:   AssetInfo,
    pub base_index:    u32,
    pub quote_index:   u32,
}




#[cw_serde]
pub struct PopulatedHopInfo {
    pub address:       String,
    pub pool_type:     PoolType,
    pub base_asset:    AssetInfo,
    pub quote_asset:   AssetInfo,
    pub base_index:    u32,
    pub quote_index:   u32,
}