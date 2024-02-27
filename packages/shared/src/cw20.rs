use cosmwasm_std::{
    to_json_binary, Addr, Api, BankMsg, Binary, Coin, CosmosMsg, Deps, MessageInfo, StdError,
    StdResult, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::coin::one_from;

pub fn is_cw20_token(api: &dyn Api, denom: &str) -> StdResult<Addr> {
    if denom.len() > 10 {
        api.addr_validate(denom)
    } else {
        Err(StdError::ParseErr {
            target_type: "Addr".to_string(),
            msg: "Not a cw20 contract address".to_string(),
        })
    }
}

pub fn from_cw20(
    deps: &Deps,
    info: MessageInfo,
    receive_msg: Cw20ReceiveMsg,
) -> StdResult<MessageInfo> {
    Ok(MessageInfo {
        sender: deps.api.addr_validate(receive_msg.sender.as_ref())?,
        funds: vec![Coin {
            amount: receive_msg.amount,
            denom: deps.api.addr_validate(info.sender.as_ref())?.to_string(),
        }],
    })
}

pub fn into_bank_msg(api: &dyn Api, recipient: &str, amount: Vec<Coin>) -> StdResult<CosmosMsg> {
    let token = one_from(amount.clone())?;
    Ok(match is_cw20_token(api, token.denom.as_ref()) {
        Ok(token_address) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: api.addr_validate(recipient)?.to_string(),
                amount: token.amount,
            })?,
            funds: vec![],
        }),
        Err(_) => CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount,
        }),
    })
}

pub fn into_execute_msg(
    api: &dyn Api,
    contract_address: Addr,
    msg: Binary,
    amount: Coin,
) -> StdResult<CosmosMsg> {
    Ok(match is_cw20_token(api, amount.denom.as_ref()) {
        Ok(token_address) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: contract_address.to_string(),
                amount: amount.amount,
                msg,
            })?,
            funds: vec![],
        }),
        Err(_) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_address.to_string(),
            msg,
            funds: vec![amount],
        }),
    })
}
