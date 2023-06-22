use cosmwasm_std::{Decimal256, Deps, MessageInfo, Reply, Response};

use crate::ContractError;

pub fn submit_order_handler(
    _deps: Deps,
    _info: MessageInfo,
    _target_price: Decimal256,
    _target_denom: String,
) -> Result<Response, ContractError> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}

pub fn return_order_idx(_reply: Reply) -> Result<Response, ContractError> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}
