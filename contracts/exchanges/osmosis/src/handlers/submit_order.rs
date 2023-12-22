use cosmwasm_std::{Decimal256, Deps, MessageInfo, Reply, Response};

use crate::ContractError;

pub fn submit_order_handler(
    _deps: Deps,
    _info: MessageInfo,
    _target_price: Decimal256,
    _target_denom: String,
) -> Result<Response, ContractError> {
    unimplemented!("limit orders not yet supported")
}

pub fn return_order_idx(_reply: Reply) -> Result<Response, ContractError> {
    unimplemented!("Limit orders are not supported on osmosis yet")
}

// use cosmwasm_std::{Decimal256, Deps, MessageInfo, Reply, Response, SubMsg, WasmMsg};
// use exchange::msg::ExecuteMsg;

// use crate::{contract::AFTER_SUBMIT_ORDER, state::config::get_config, ContractError};

// pub enum TFMExecuteMsg {
//     SubmitOrder {
//         ask_asset: AssetInfo;
//         offer_asset: AssetInfo;
//         expiration_time: u64;
//     }
// }

// pub fn submit_order_handler(
//     deps: Deps,
//     info: MessageInfo,
//     target_price: Decimal256,
//     target_denom: String,
// ) -> Result<Response, ContractError> {
//     if info.funds.len() != 1 {
//         return Err(ContractError::InvalidFunds {
//             msg: String::from("must send exactly one asset"),
//         });
//     }

//     if info.funds[0].denom.clone() == target_denom {
//         return Err(ContractError::InvalidFunds {
//             msg: String::from("swap denom and target denom must be different"),
//         });
//     }

//     let config = get_config(deps.storage)?;

//     if info.sender != config.dca_contract_address {
//         return Err(ContractError::Unauthorized {});
//     }

//     Ok(Response::new()
//         .add_attribute("submit_order", "true")
//         .add_attribute("target_price", target_price.to_string())
//         .add_submessage(SubMsg::reply_on_success(
//             WasmMsg::Execute {
//                 contract_addr: config.limit_order_address.to_string(),
//                 msg: ExecuteMsg::SubmitOrder {
//                     target_price,
//                     target_denom,
//                 },
//                 funds: info.funds,
//             },
//             AFTER_SUBMIT_ORDER,
//         )))
// }

// pub fn return_order_idx(reply: Reply) -> Result<Response, ContractError> {
//     let order_idx = get_attribute_in_event(
//         &reply.result.into_result().unwrap().events,
//         "wasm",
//         "order_idx",
//     )?
//     .parse::<Uint128>()
//     .unwrap();

//     Ok(Response::new().add_attribute("order_idx", order_idx))
// }
