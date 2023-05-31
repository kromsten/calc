#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use exchange::msg::{ExecuteMsg, QueryMsg};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::create_pairs::create_pairs_handler;
use crate::handlers::submit_order::{return_order_idx, submit_order_handler};
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
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&msg.admin.to_string())?;
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitOrder {
            target_price,
            target_denom,
        } => submit_order_handler(deps.as_ref(), info, target_price, target_denom),
        ExecuteMsg::InternalMsg(msg) => match from_binary(&msg).unwrap() {
            InternalMsg::CreatePairs { pairs } => create_pairs_handler(deps, info, pairs),
        },
        _ => unimplemented!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

pub const AFTER_SUBMIT_ORDER: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_SUBMIT_ORDER => return_order_idx(reply),
        _ => Err(ContractError::MissingReplyId {}),
    }
}
