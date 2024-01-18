use cosmwasm_std::{to_json_binary, BankMsg, Coin, CosmosMsg, Event, QuerierWrapper, StdError, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use astrovault::
    assets::asset::{Asset, AssetInfo}
;

use crate::types::{wrapper::ContractWrapper, pair::Pair};

use super::pool;


pub fn get_attribute_in_event(
    events: &[Event],
    event_type: &str,
    attribute_key: &str,
) -> StdResult<String> {
    let events_with_type = events.iter().filter(|event| event.ty == event_type);

    let attribute = events_with_type
        .into_iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attribute| attribute.key == attribute_key)
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "unable to find {} attribute in {} event",
                attribute_key, event_type
            ))
        })?;

    Ok(attribute.value.clone())
}


pub fn send_asset_msg(
    recipient: String,
    info: AssetInfo,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    match info {
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient,
                amount,
            })?,
            funds: vec![],
        })),
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient,
            amount: vec![Coin { denom, amount }],
        })),
    }
}

pub fn get_swap_msg(
    querier: &QuerierWrapper,
    pair: &Pair,
    offer_asset: Asset,
    min_amount: Asset,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {

    let msg = if pair.is_pool_pair() {
        
        let address = pair.address.clone().unwrap();
        let pool_type = pair.pool_type.as_ref().unwrap();

        let pool_msg = pool::swap_msg(
            querier, 
            address.as_ref(),
            pool_type,
            offer_asset.clone(), 
            min_amount
        )?;

        let pool_contract  = ContractWrapper(address);

        if offer_asset.is_native_token() {
            pool_contract.execute(pool_msg, funds)?
        } else {
            pool_contract.execute_cw20(
                offer_asset.info.to_string(), 
                offer_asset.amount, 
                pool_msg
            )?
        }

    } else {
        todo!()
    };

    Ok(msg)
}