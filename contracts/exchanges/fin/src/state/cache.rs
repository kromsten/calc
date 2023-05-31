use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct SwapCache {
    pub sender: Addr,
    pub minimum_receive_amount: Coin,
    pub target_denom_balance: Coin,
}

pub const SWAP_CACHE: Item<SwapCache> = Item::new("swap_cache_v8");
