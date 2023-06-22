use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::ContractError;

pub fn retract_order_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> Result<Response, ContractError> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}

pub fn return_retracted_funds(_deps: Deps, _env: Env) -> Result<Response, ContractError> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}
