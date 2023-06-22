use cosmwasm_std::{Deps, StdResult, Uint128};
use exchange::msg::Order;

pub fn get_order_handler(
    _deps: Deps,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> StdResult<Order> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}
