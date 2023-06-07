use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct Order {
    pub order_idx: Uint128,
    pub remaining_offer_amount: Coin,
}
