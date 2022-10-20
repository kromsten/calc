use crate::contract::{
    FIN_LIMIT_ORDER_WITHDRAWN_FOR_EXECUTE_VAULT_ID, FIN_SWAP_COMPLETED_ID,
};
use crate::error::ContractError;
use crate::state::{
    create_event, get_trigger, vault_store, Cache, LimitOrderCache, CACHE,
    LIMIT_ORDER_CACHE,
};
use crate::vault::Vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::time_helpers::target_time_elapsed;
use base::triggers::trigger::TriggerConfiguration;
use base::vaults::vault::{PositionType, VaultStatus};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_sub_msg;
use fin_helpers::queries::{query_base_price, query_order_details, query_quote_price};
use fin_helpers::swaps::{create_fin_swap_with_slippage, create_fin_swap_without_slippage};

pub fn execute_trigger(
    deps: DepsMut,
    env: Env,
    trigger_id: Uint128,
) -> Result<Response, ContractError> {
    let trigger = get_trigger(deps.storage, trigger_id.into())?;
    let vault = vault_store().load(deps.storage, trigger.vault_id.into())?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.to_owned(),
            EventData::DCAVaultExecutionTriggered,
        ),
    )?;

    let response = Response::new().add_attribute("method", "execute_trigger");

    match trigger.configuration {
        TriggerConfiguration::Time { target_time } => {
            if !target_time_elapsed(env.block.time, target_time) {
                return Err(ContractError::CustomError {
                    val: String::from("trigger execution time has not yet elapsed"),
                });
            }

            if vault.low_funds() {
                vault_store().update(
                    deps.storage,
                    vault.id.into(),
                    |existing_vault| -> Result<Vault, ContractError> {
                        match existing_vault {
                            Some(mut existing_vault) => {
                                existing_vault.status = VaultStatus::Inactive;
                                Ok(existing_vault)
                            }
                            None => Err(ContractError::CustomError {
                                val: format!(
                                    "could not find vault for address: {} with id: {}",
                                    vault.owner.clone(),
                                    vault.id
                                ),
                            }),
                        }
                    },
                )?;
            }

            let fin_swap_msg = match vault.slippage_tolerance {
                Some(tolerance) => {
                    let belief_price = match vault.position_type {
                        PositionType::Enter => {
                            query_base_price(deps.querier, vault.pair.address.clone())
                        }
                        PositionType::Exit => {
                            query_quote_price(deps.querier, vault.pair.address.clone())
                        }
                    };

                    create_fin_swap_with_slippage(
                        vault.pair.address.clone(),
                        belief_price,
                        tolerance,
                        vault.get_swap_amount(),
                        FIN_SWAP_COMPLETED_ID,
                    )
                }
                None => create_fin_swap_without_slippage(
                    vault.pair.address.clone(),
                    vault.get_swap_amount(),
                    FIN_SWAP_COMPLETED_ID,
                ),
            };

            CACHE.save(
                deps.storage,
                &Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                },
            )?;

            Ok(response.add_submessage(fin_swap_msg))
        }
        TriggerConfiguration::FINLimitOrder { order_idx, .. } => {
            let (offer_amount, original_offer_amount, filled) =
                query_order_details(deps.querier, vault.pair.address.clone(), order_idx.unwrap());

            let limit_order_cache = LimitOrderCache {
                offer_amount,
                original_offer_amount,
                filled,
            };

            LIMIT_ORDER_CACHE.save(deps.storage, &limit_order_cache)?;

            if offer_amount != Uint128::zero() {
                return Err(ContractError::CustomError {
                    val: String::from("fin limit order has not been completely filled"),
                });
            }

            let fin_withdraw_sub_msg = create_withdraw_limit_order_sub_msg(
                vault.pair.address,
                order_idx.unwrap(),
                FIN_LIMIT_ORDER_WITHDRAWN_FOR_EXECUTE_VAULT_ID,
            );

            let cache: Cache = Cache {
                vault_id: vault.id,
                owner: vault.owner.clone(),
            };

            CACHE.save(deps.storage, &cache)?;

            Ok(response.add_submessage(fin_withdraw_sub_msg))
        }
    }
}
