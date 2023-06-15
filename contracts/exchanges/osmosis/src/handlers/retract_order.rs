use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::ContractError;

pub fn retract_order_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn return_retracted_funds(_deps: Deps, _env: Env) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod retract_order_handler_tests {

    #[test]
    fn with_funds_fails() {
        unimplemented!()
    }

    #[test]
    fn caches_sender_and_pair_balances() {
        unimplemented!()
    }

    #[test]
    fn sends_withdraw_order_message() {
        unimplemented!()
    }
}

#[cfg(test)]
mod return_retracted_funds_tests {

    #[test]
    fn returns_funds_difference_to_sender() {
        unimplemented!()
    }

    #[test]
    fn drops_empty_funds_differences() {
        unimplemented!()
    }

    #[test]
    fn with_no_differences_drops_bank_send_message() {
        unimplemented!()
    }
}
