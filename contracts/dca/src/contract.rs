use crate::constants::{
    AFTER_DELEGATION_REPLY_ID, AFTER_FAILED_AUTOMATION_REPLY_ID, AFTER_LIMIT_ORDER_PLACED_REPLY_ID,
    AFTER_SWAP_REPLY_ID,
};
use crate::error::ContractError;
use crate::handlers::cancel_vault::cancel_vault_handler;
use crate::handlers::create_custom_swap_fee::create_custom_swap_fee_handler;
use crate::handlers::create_pair::create_pair_handler;
use crate::handlers::create_vault::create_vault_handler;
use crate::handlers::delete_pair::delete_pair_handler;
use crate::handlers::deposit::deposit_handler;
use crate::handlers::disburse_escrow::disburse_escrow_handler;
use crate::handlers::disburse_funds::disburse_funds_handler;
use crate::handlers::execute_trigger::execute_trigger_handler;
use crate::handlers::get_config::get_config_handler;
use crate::handlers::get_custom_swap_fees::get_custom_swap_fees_handler;
use crate::handlers::get_disburse_escrow_tasks::get_disburse_escrow_tasks_handler;
use crate::handlers::get_events::get_events_handler;
use crate::handlers::get_events_by_resource_id::get_events_by_resource_id_handler;
use crate::handlers::get_pairs::get_pairs_handler;
use crate::handlers::get_time_trigger_ids::get_time_trigger_ids_handler;
use crate::handlers::get_trigger_id_by_fin_limit_order_idx::get_trigger_id_by_fin_limit_order_idx_handler;
use crate::handlers::get_vault::get_vault_handler;
use crate::handlers::get_vault_performance::get_vault_performance_handler;
use crate::handlers::get_vaults::get_vaults_handler;
use crate::handlers::get_vaults_by_address::get_vaults_by_address_handler;
use crate::handlers::handle_failed_automation::handle_failed_automation_handler;
use crate::handlers::instantiate::instantiate_handler;
use crate::handlers::migrate::migrate_handler;
use crate::handlers::migrate_vaults::migrate_vaults_handler;
use crate::handlers::remove_custom_swap_fee::remove_custom_swap_fee_handler;
use crate::handlers::save_limit_order_id::save_limit_order_id;
use crate::handlers::update_config::update_config_handler;
use crate::handlers::update_swap_adjustment_handler::update_swap_adjustment_handler;
use crate::handlers::update_vault::update_vault_handler;
use crate::handlers::z_delegate::{log_delegation_result, z_delegate_handler};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

pub const CONTRACT_NAME: &str = "crates.io:calc-dca";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_handler(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    instantiate_handler(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePair {
            base_denom,
            quote_denom,
            address,
        } => create_pair_handler(deps, info, base_denom, quote_denom, address),
        ExecuteMsg::DeletePair { denoms } => delete_pair_handler(deps, denoms),
        ExecuteMsg::CreateVault {
            owner,
            label,
            destinations,
            target_denom,
            position_type,
            slippage_tolerance,
            minimum_receive_amount,
            swap_amount,
            time_interval,
            target_start_time_utc_seconds,
            target_receive_amount,
            performance_assessment_strategy,
            swap_adjustment_strategy,
        } => create_vault_handler(
            deps,
            env,
            &info,
            owner.unwrap_or_else(|| info.sender.clone()),
            label,
            destinations.unwrap_or_default(),
            target_denom,
            position_type,
            slippage_tolerance,
            minimum_receive_amount,
            swap_amount,
            time_interval,
            target_start_time_utc_seconds,
            target_receive_amount,
            performance_assessment_strategy,
            swap_adjustment_strategy,
        ),
        ExecuteMsg::UpdateVault {
            vault_id,
            label,
            destinations,
        } => update_vault_handler(deps, info, vault_id, label, destinations),
        ExecuteMsg::CancelVault { vault_id } => cancel_vault_handler(deps, env, info, vault_id),
        ExecuteMsg::ExecuteTrigger { trigger_id } => execute_trigger_handler(deps, env, trigger_id),
        ExecuteMsg::Deposit { address, vault_id } => {
            deposit_handler(deps, env, info, address, vault_id)
        }
        ExecuteMsg::UpdateConfig {
            executors,
            fee_collectors,
            default_swap_fee_percent,
            weighted_scale_swap_fee_percent,
            automation_fee_percent,
            default_page_limit,
            paused,
            risk_weighted_average_escrow_level,
            twap_period,
            default_slippage_tolerance,
        } => update_config_handler(
            deps,
            info,
            executors,
            fee_collectors,
            default_swap_fee_percent,
            weighted_scale_swap_fee_percent,
            automation_fee_percent,
            default_page_limit,
            paused,
            risk_weighted_average_escrow_level,
            twap_period,
            default_slippage_tolerance,
        ),
        ExecuteMsg::CreateCustomSwapFee {
            denom,
            swap_fee_percent,
        } => create_custom_swap_fee_handler(deps, info, denom, swap_fee_percent),
        ExecuteMsg::RemoveCustomSwapFee { denom } => {
            remove_custom_swap_fee_handler(deps, info, denom)
        }
        ExecuteMsg::UpdateSwapAdjustment { strategy, value } => {
            update_swap_adjustment_handler(deps, env, info, strategy, value)
        }
        ExecuteMsg::DisburseEscrow { vault_id } => {
            disburse_escrow_handler(deps, &env, info, vault_id)
        }
        ExecuteMsg::ZDelegate {
            delegator_address,
            validator_address,
        } => z_delegate_handler(
            deps.as_ref(),
            env,
            info,
            delegator_address,
            validator_address,
        ),
        ExecuteMsg::MigrateVaults { limit } => migrate_vaults_handler(deps, env, limit),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_LIMIT_ORDER_PLACED_REPLY_ID => save_limit_order_id(deps, reply),
        AFTER_SWAP_REPLY_ID => disburse_funds_handler(deps, &env, reply),
        AFTER_FAILED_AUTOMATION_REPLY_ID => handle_failed_automation_handler(deps, env, reply),
        AFTER_DELEGATION_REPLY_ID => log_delegation_result(reply),
        id => Err(ContractError::CustomError {
            val: format!("unhandled DCA contract reply id: {}", id),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPairs {} => to_binary(&get_pairs_handler(deps)?),
        QueryMsg::GetTimeTriggerIds { limit } => {
            to_binary(&get_time_trigger_ids_handler(deps, env, limit)?)
        }
        QueryMsg::GetTriggerIdByFinLimitOrderIdx { order_idx } => to_binary(
            &get_trigger_id_by_fin_limit_order_idx_handler(deps, order_idx)?,
        ),
        QueryMsg::GetVaults {
            start_after,
            limit,
            reverse,
        } => to_binary(&get_vaults_handler(deps, start_after, limit, reverse)?),
        QueryMsg::GetVaultsByAddress {
            address,
            status,
            start_after,
            limit,
        } => to_binary(&get_vaults_by_address_handler(
            deps,
            address,
            status,
            start_after,
            limit,
        )?),
        QueryMsg::GetVault { vault_id } => to_binary(&get_vault_handler(deps, vault_id)?),
        QueryMsg::GetEventsByResourceId {
            resource_id,
            start_after,
            limit,
            reverse,
        } => to_binary(&get_events_by_resource_id_handler(
            deps,
            resource_id,
            start_after,
            limit,
            reverse,
        )?),
        QueryMsg::GetEvents {
            start_after,
            limit,
            reverse,
        } => to_binary(&get_events_handler(deps, start_after, limit, reverse)?),
        QueryMsg::GetCustomSwapFees {} => to_binary(&get_custom_swap_fees_handler(deps)?),
        QueryMsg::GetConfig {} => to_binary(&get_config_handler(deps)?),
        QueryMsg::GetVaultPerformance { vault_id } => {
            to_binary(&get_vault_performance_handler(deps, vault_id)?)
        }
        QueryMsg::GetDisburseEscrowTasks { limit } => {
            to_binary(&get_disburse_escrow_tasks_handler(deps, env, limit)?)
        }
    }
}
