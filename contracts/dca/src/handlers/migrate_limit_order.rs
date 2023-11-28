use cosmwasm_std::{to_json_binary, Coin, DepsMut, Reply, Response, SubMsg, Uint128, WasmMsg};
use exchange::msg::ExecuteMsg;

use crate::{
    constants::{AFTER_ORDER_MIGRATION_REPLY_ID, FAIL_SILENTLY_REPLY_ID, TWO_MICRONS},
    error::ContractError,
    helpers::message::get_attribute_in_event,
    state::{
        cache::VAULT_ID_CACHE,
        config::get_config,
        triggers::{delete_trigger, save_trigger},
        vaults::get_vault,
    },
    types::trigger::{Trigger, TriggerConfiguration},
};

pub fn migrate_limit_order(deps: DepsMut, vault_id: Uint128) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, vault_id)?;

    let mut response = Response::new()
        .add_attribute("migrate_limit_order", "true")
        .add_attribute("vault_id", vault_id.to_string());

    if let Some(TriggerConfiguration::Price {
        target_price,
        order_idx,
    }) = vault.trigger
    {
        VAULT_ID_CACHE.save(deps.storage, &vault_id)?;

        let config = get_config(deps.storage)?;

        response = response.add_submessage(SubMsg::reply_on_error(
            WasmMsg::Execute {
                contract_addr: config.exchange_contract_address.to_string(),
                msg: to_json_binary(&ExecuteMsg::RetractOrder {
                    order_idx,
                    denoms: vault.denoms(),
                })
                .unwrap(),
                funds: vec![],
            },
            FAIL_SILENTLY_REPLY_ID,
        ));

        response = response.add_submessage(SubMsg::reply_on_error(
            WasmMsg::Execute {
                contract_addr: config.exchange_contract_address.to_string(),
                msg: to_json_binary(&ExecuteMsg::WithdrawOrder {
                    order_idx,
                    denoms: vault.denoms(),
                })
                .unwrap(),
                funds: vec![],
            },
            FAIL_SILENTLY_REPLY_ID,
        ));

        response = response.add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: config.exchange_contract_address.to_string(),
                msg: to_json_binary(&ExecuteMsg::SubmitOrder {
                    target_price: target_price.into(),
                    target_denom: vault.target_denom.clone(),
                })
                .unwrap(),
                funds: vec![Coin::new(TWO_MICRONS.into(), vault.get_swap_denom())],
            },
            AFTER_ORDER_MIGRATION_REPLY_ID,
        ))
    }

    Ok(response)
}

pub fn save_new_limit_order_idx(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let order_idx = get_attribute_in_event(
        &reply.result.into_result().unwrap().events,
        "wasm",
        "order_idx",
    )?
    .parse::<Uint128>()
    .expect("the order id of the new limit order");

    let vault_id = VAULT_ID_CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, vault_id)?;

    if let Some(TriggerConfiguration::Price { target_price, .. }) = vault.trigger {
        delete_trigger(deps.storage, vault_id)?;

        save_trigger(
            deps.storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Price {
                    target_price,
                    order_idx,
                },
            },
        )?;
    }

    Ok(Response::new().add_attribute("order_idx", order_idx))
}
