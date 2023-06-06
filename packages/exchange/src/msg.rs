use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Decimal256, Uint128};

use crate::pair::Pair;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        minimum_receive_amount: Coin,
    },
    SubmitOrder {
        target_price: Decimal256,
        target_denom: String,
    },
    RetractOrder {
        order_idx: Uint128,
        denoms: [String; 2],
    },
    WithdrawOrder {
        order_idx: Uint128,
        denoms: [String; 2],
    },
    InternalMsg {
        msg: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Pair>)]
    GetPairs {
        limit: u16,
        start_after: Option<Pair>,
    },
    #[returns(OrderStatus)]
    GetOrderStatus { order_idx: Uint128 },
}

#[cw_serde]
pub enum OrderStatus {
    Unfilled,
    Filled,
}

#[cw_serde]
pub struct Order {
    pub order_idx: Uint128,
    pub original_offer_amount: Coin,
    pub remaining_offer_amount: Coin,
    pub filled_amount: Coin,
}
