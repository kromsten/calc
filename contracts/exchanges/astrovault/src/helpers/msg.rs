use cosmwasm_std::{
    to_json_binary, BankMsg, Coin, CosmosMsg, Event, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use astrovault::assets::asset::AssetInfo;

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

pub fn send_asset_msg(recipient: String, info: AssetInfo, amount: Uint128) -> StdResult<CosmosMsg> {
    match info {
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer { recipient, amount })?,
            funds: vec![],
        })),
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient,
            amount: vec![Coin { denom, amount }],
        })),
    }
}
