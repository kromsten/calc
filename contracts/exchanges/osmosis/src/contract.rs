#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use exchange::msg::{ExecuteMsg, QueryMsg};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::create_pairs::create_pairs_handler;
use crate::handlers::get_expected_receive_amount::get_expected_receive_amount_handler;
use crate::handlers::get_order::get_order_handler;
use crate::handlers::get_pairs::get_pairs_handler;
use crate::handlers::get_twap_to_now::get_twap_to_now_handler;
use crate::handlers::retract_order::{retract_order_handler, return_retracted_funds};
use crate::handlers::submit_order::{return_order_idx, submit_order_handler};
use crate::handlers::swap::{return_swapped_funds, swap_handler};
use crate::handlers::withdraw_order::{return_withdrawn_funds, withdraw_order_handler};
use crate::msg::{InstantiateMsg, InternalMsg};
use crate::state::config::update_config;
use crate::types::config::Config;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fin";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(msg.admin.as_ref())?;
    update_config(
        deps.storage,
        Config {
            admin: msg.admin.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("instantiate", "true")
        .add_attribute("admin", msg.admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            minimum_receive_amount,
        } => swap_handler(deps, env, info, minimum_receive_amount),
        ExecuteMsg::SubmitOrder {
            target_price,
            target_denom,
        } => submit_order_handler(deps.as_ref(), info, target_price, target_denom),
        ExecuteMsg::RetractOrder { order_idx, denoms } => {
            retract_order_handler(deps, env, info, order_idx, denoms)
        }
        ExecuteMsg::WithdrawOrder { order_idx, denoms } => {
            withdraw_order_handler(deps, env, info, order_idx, denoms)
        }
        ExecuteMsg::InternalMsg { msg } => match from_binary(&msg).unwrap() {
            InternalMsg::CreatePairs { pairs } => create_pairs_handler(deps, info, pairs),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPairs { start_after, limit } => {
            to_binary(&get_pairs_handler(deps, start_after, limit)?)
        }
        QueryMsg::GetOrder { order_idx, denoms } => {
            to_binary(&get_order_handler(deps, order_idx, denoms)?)
        }
        QueryMsg::GetTwapToNow {
            swap_denom,
            target_denom,
            period,
        } => to_binary(&get_twap_to_now_handler(
            deps,
            env,
            swap_denom,
            target_denom,
            period,
        )?),
        QueryMsg::GetExpectedReceiveAmount {
            swap_amount,
            target_denom,
        } => to_binary(&get_expected_receive_amount_handler(
            deps,
            env,
            swap_amount,
            target_denom,
        )?),
    }
}

pub const AFTER_SWAP: u64 = 1;
pub const AFTER_SUBMIT_ORDER: u64 = 2;
pub const AFTER_RETRACT_ORDER: u64 = 3;
pub const AFTER_WITHDRAW_ORDER: u64 = 4;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_SWAP => return_swapped_funds(deps.as_ref(), env),
        AFTER_SUBMIT_ORDER => return_order_idx(reply),
        AFTER_RETRACT_ORDER => return_retracted_funds(deps.as_ref(), env),
        AFTER_WITHDRAW_ORDER => return_withdrawn_funds(deps.as_ref(), env),
        _ => Err(ContractError::MissingReplyId {}),
    }
}
