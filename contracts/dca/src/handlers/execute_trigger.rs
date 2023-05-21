use crate::contract::AFTER_FIN_SWAP_REPLY_ID;
use crate::error::ContractError;
use crate::helpers::validation_helpers::{
    assert_contract_is_not_paused, assert_target_time_is_in_past,
};
use crate::helpers::vault_helpers::{
    get_swap_amount, price_threshold_exceeded, simulate_standard_dca_execution,
};
use crate::msg::ExecuteMsg;
use crate::state::events::create_event;
use crate::state::old_cache::{Cache, SwapCache, OLD_CACHE, OLD_SWAP_CACHE};
use crate::state::old_triggers::{delete_old_trigger, save_old_trigger};
use crate::state::old_vaults::{get_old_vault, update_old_vault};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{OldTrigger, OldTriggerConfiguration};
use base::vaults::vault::OldVaultStatus;
use cosmwasm_std::{to_binary, CosmosMsg, ReplyOn, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_msg;
use fin_helpers::queries::{query_belief_price, query_order_details};
use fin_helpers::swaps::create_fin_swap_message;

pub fn execute_trigger_handler(
    deps: DepsMut,
    env: Env,
    trigger_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    let mut response = Response::new().add_attribute("method", "execute_trigger");
    let mut vault = get_old_vault(deps.storage, trigger_id.into())?;

    delete_old_trigger(deps.storage, vault.id)?;

    if vault.is_cancelled() {
        return Err(ContractError::CustomError {
            val: format!(
                "vault with id {} is cancelled, and is not available for execution",
                vault.id
            ),
        });
    }

    if vault.trigger.is_none() {
        return Err(ContractError::CustomError {
            val: format!(
                "vault with id {} has no trigger attached, and is not available for execution",
                vault.id
            ),
        });
    }

    match vault
        .trigger
        .clone()
        .expect(format!("trigger for vault id {}", vault.id).as_str())
    {
        OldTriggerConfiguration::Time { target_time } => {
            assert_target_time_is_in_past(env.block.time, target_time)?;
        }
        OldTriggerConfiguration::FinLimitOrder { order_idx, .. } => {
            if let Some(order_idx) = order_idx {
                let limit_order =
                    query_order_details(deps.querier, vault.pair.address.clone(), order_idx)?;

                if limit_order.offer_amount != Uint128::zero() {
                    return Err(ContractError::CustomError {
                        val: String::from("fin limit order has not been completely filled"),
                    });
                }

                if limit_order.filled_amount > Uint128::zero() {
                    response = response.add_message(create_withdraw_limit_order_msg(
                        vault.pair.address.clone(),
                        order_idx,
                    ));
                }
            } else {
                return Err(ContractError::CustomError {
                    val: String::from("fin limit order has not been created"),
                });
            }
        }
    }

    if vault.is_scheduled() {
        vault.status = OldVaultStatus::Active;
        vault.started_at = Some(env.block.time);
    }

    update_old_vault(deps.storage, &vault)?;

    let belief_price = query_belief_price(&deps.querier, &vault.pair, &vault.get_swap_denom())?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.to_owned(),
            EventData::DcaVaultExecutionTriggered {
                base_denom: vault.pair.base_denom.clone(),
                quote_denom: vault.pair.quote_denom.clone(),
                asset_price: belief_price.clone(),
            },
        ),
    )?;

    if vault.is_dca_plus() {
        vault = simulate_standard_dca_execution(
            &deps.querier,
            deps.storage,
            &env,
            vault,
            belief_price,
        )?;
    }

    let should_execute_again = vault.is_active()
        || vault
            .dca_plus_config
            .clone()
            .map_or(false, |dca_plus_config| {
                dca_plus_config.has_sufficient_funds()
            });

    if should_execute_again {
        save_old_trigger(
            deps.storage,
            OldTrigger {
                vault_id: vault.id,
                configuration: OldTriggerConfiguration::Time {
                    target_time: get_next_target_time(
                        env.block.time,
                        match vault.trigger {
                            Some(OldTriggerConfiguration::Time { target_time }) => target_time,
                            _ => env.block.time,
                        },
                        vault.time_interval.clone(),
                    ),
                },
            },
        )?;
    } else {
        if vault.is_finished_dca_plus_vault() {
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::DisburseEscrow { vault_id: vault.id })?,
                funds: vec![],
            }));
        }

        return Ok(response);
    }

    if vault.is_inactive() {
        return Ok(response);
    }

    let swap_amount = get_swap_amount(&deps.as_ref(), &env, &vault)?;

    if price_threshold_exceeded(
        swap_amount.amount,
        vault.minimum_receive_amount,
        belief_price,
    )? {
        create_event(
            deps.storage,
            EventBuilder::new(
                vault.id,
                env.block.to_owned(),
                EventData::DcaVaultExecutionSkipped {
                    reason: ExecutionSkippedReason::PriceThresholdExceeded {
                        price: belief_price,
                    },
                },
            ),
        )?;

        return Ok(response.to_owned());
    };

    OLD_CACHE.save(
        deps.storage,
        &Cache {
            vault_id: vault.id,
            owner: vault.owner.clone(),
        },
    )?;

    OLD_SWAP_CACHE.save(
        deps.storage,
        &SwapCache {
            swap_denom_balance: deps
                .querier
                .query_balance(&env.contract.address, &vault.get_swap_denom())?,
            receive_denom_balance: deps
                .querier
                .query_balance(&env.contract.address, &vault.get_receive_denom())?,
        },
    )?;

    Ok(response.add_submessage(create_fin_swap_message(
        &deps.querier,
        vault.pair.clone(),
        swap_amount,
        vault.slippage_tolerance,
        Some(AFTER_FIN_SWAP_REPLY_ID),
        Some(ReplyOn::Always),
    )?))
}
