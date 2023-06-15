use cosmwasm_std::{Decimal256, Deps, MessageInfo, Reply, Response};

use crate::ContractError;

pub fn submit_order_handler(
    _deps: Deps,
    _info: MessageInfo,
    _target_price: Decimal256,
    _target_denom: String,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn return_order_idx(_reply: Reply) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod submit_order_tests {

    #[test]
    fn with_no_assets_fails() {
        unimplemented!()
    }

    #[test]
    fn with_more_than_one_asset_fails() {
        unimplemented!()
    }

    #[test]
    fn with_the_same_swap_and_target_denom_fails() {
        unimplemented!()
    }

    #[test]
    fn with_no_matching_pair_fails() {
        unimplemented!()
    }

    #[test]
    fn sends_submit_order_message() {
        unimplemented!()
    }

    #[test]
    fn inverts_price_for_fin_sell() {
        unimplemented!()
    }
}
