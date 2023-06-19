use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Decimal, Decimal256, Uint128};

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
        limit: Option<u16>,
        start_after: Option<Pair>,
    },
    #[returns(Order)]
    GetOrder {
        order_idx: Uint128,
        denoms: [String; 2],
    },
    #[returns(Decimal)]
    GetTwapToNow {
        swap_denom: String,
        target_denom: String,
        period: u64,
    },
    #[returns(Coin)]
    GetExpectedReceiveAmount {
        swap_amount: Coin,
        target_denom: String,
    },
}

#[cw_serde]
pub struct Pair {
    pub denoms: [String; 2],
}

impl Pair {
    pub fn other_denom(self, denom: String) -> String {
        if self.denoms[0] == denom {
            self.denoms[1].clone()
        } else {
            self.denoms[0].clone()
        }
    }
}

impl Default for Pair {
    fn default() -> Self {
        Pair {
            denoms: ["uusd".to_string(), "uatom".to_string()],
        }
    }
}

#[cw_serde]
pub struct Order {
    pub order_idx: Uint128,
    pub remaining_offer_amount: Coin,
}
