use astrovault::assets::asset::Asset;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct SwapCache {
    pub sender: Addr,
    pub minimum_receive_amount: Asset,
    pub target_asset_balance: Asset,
}


pub const SWAP_CACHE: Item<SwapCache> = Item::new("swap_cache_v1");
