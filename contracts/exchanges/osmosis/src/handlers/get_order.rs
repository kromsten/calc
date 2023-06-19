use cosmwasm_std::{Deps, StdResult, Uint128};
use exchange::msg::Order;

pub fn get_order_handler(
    _deps: Deps,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> StdResult<Order> {
    unimplemented!()
}

#[cfg(test)]
mod get_order_handler_tests {

    #[test]
    fn for_missing_pair_fails() {
        unimplemented!()
    }

    #[test]
    fn for_missing_order_fails() {
        unimplemented!()
    }

    #[test]
    fn for_valid_order_returns_order() {
        unimplemented!()
    }
}
