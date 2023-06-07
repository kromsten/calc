use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Uint256};

#[cw_serde]
pub struct Order {
    pub order_idx: Uint128,
    pub original_offer_amount: Uint256,
    pub remaining_offer_amount: Uint256,
    pub filled_amount: Uint256,
}
