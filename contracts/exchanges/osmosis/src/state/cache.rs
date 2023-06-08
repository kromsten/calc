use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct SwapCache {
    pub sender: Addr,
    pub minimum_receive_amount: Coin,
    pub target_denom_balance: Coin,
}

pub const SWAP_CACHE: Item<SwapCache> = Item::new("swap_cache_v1");

#[cw_serde]
pub struct LimitOrderCache {
    pub sender: Addr,
    pub balances: HashMap<String, Coin>,
}

pub const LIMIT_ORDER_CACHE: Item<LimitOrderCache> = Item::new("limit_order_cache_v1");
