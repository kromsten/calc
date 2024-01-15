#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, from_json, to_json_binary,
};
// use cw2::set_contract_version;

use crate::helpers::balance::coin_to_asset;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::error::ContractError;
use crate::handlers::create_pairs::create_pairs_handler;
use crate::handlers::get_expected_receive_amount::get_expected_receive_amount_handler;
use crate::handlers::get_pairs::get_pairs_handler;
use crate::handlers::get_pairs_internal::get_pairs_internal_handler;
use crate::handlers::get_twap_to_now::get_twap_to_now_handler;
use crate::handlers::swap::{return_swapped_funds, swap_native_handler, swap_cw20_handler};
use crate::msg::{InstantiateMsg, InternalExecuteMsg, InternalQueryMsg, MigrateMsg};
use crate::state::config::update_config;
use crate::types::config::Config;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:astrovault";
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
    deps.api.addr_validate(msg.dca_contract_address.as_ref())?;

    update_config(
        deps.storage,
        Config {
            admin: msg.admin.clone(),
            dca_contract_address: msg.dca_contract_address.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("instantiate", "true")
        .add_attribute("admin", msg.admin)
        .add_attribute("dca_contract_address", msg.dca_contract_address))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    deps.api.addr_validate(msg.dca_contract_address.as_ref())?;

    update_config(
        deps.storage,
        Config {
            admin: msg.admin.clone(),
            dca_contract_address: msg.dca_contract_address.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("migrate", "true")
        .add_attribute("admin", msg.admin)
        .add_attribute("dca_contract_address", msg.dca_contract_address))
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
        } => swap_native_handler(
            deps, 
            env, 
            info, 
            coin_to_asset(minimum_receive_amount)),
        ExecuteMsg::SubmitOrder {
            target_price: _,
            target_denom: _,
        } => not_implemented_handle(),
        ExecuteMsg::RetractOrder { 
            order_idx: _, 
            denoms: _ 
        } => not_implemented_handle(),
        ExecuteMsg::WithdrawOrder { 
            order_idx: _, 
            denoms: _ 
        } => not_implemented_handle(),
        ExecuteMsg::InternalMsg { msg } => match from_json(&msg).unwrap() {
            InternalExecuteMsg::CreatePairs { pairs } => create_pairs_handler(deps, info, pairs),
        },

        ExecuteMsg::Receive(receive_msg) => {
            let msg : ExecuteMsg = from_json(&receive_msg.msg)?;
            match msg {
                ExecuteMsg::Swap { 
                    minimum_receive_amount
                } => swap_cw20_handler(
                    deps, 
                    env, 
                    info.sender, 
                    receive_msg.amount, 
                    receive_msg.sender, 
                    coin_to_asset(minimum_receive_amount)
                ),

                _ => Err(ContractError::Unauthorized {})
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPairs { start_after, limit } => {
            to_json_binary(&get_pairs_handler(deps, start_after, limit)?)
        }
        QueryMsg::GetOrder { 
            order_idx: _, 
            denoms: _ 
        } => to_json_binary(&not_implemented_query()?),
        QueryMsg::GetTwapToNow {
            swap_denom,
            target_denom,
            period,
        } => to_json_binary(&get_twap_to_now_handler(
            deps,
            swap_denom,
            target_denom,
            period,
        )?),
        QueryMsg::GetExpectedReceiveAmount {
            swap_amount,
            target_denom,
        } => to_json_binary(&get_expected_receive_amount_handler(
            deps,
            swap_amount,
            target_denom,
        )?),
        QueryMsg::InternalQuery { msg } => match from_json(&msg).unwrap() {
            InternalQueryMsg::GetPairs { start_after, limit } => {
                to_json_binary(&get_pairs_internal_handler(deps, start_after, limit)?)
            }
        },
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
        AFTER_SUBMIT_ORDER => not_implemented_handle(),
        AFTER_RETRACT_ORDER => not_implemented_handle(),
        AFTER_WITHDRAW_ORDER => not_implemented_handle(),
        _ => Err(ContractError::MissingReplyId {}),
    }
}


pub fn not_implemented_query() -> StdResult<()> {
    Err(cosmwasm_std::StdError::GenericErr { msg: "not implemented".to_string() })
}

pub fn not_implemented_handle() -> Result<Response, ContractError> {
    Err(ContractError::Std(not_implemented_query().unwrap_err()))
}