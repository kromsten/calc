use cosmwasm_std::{
    to_json_binary, Addr, Api, BalanceResponse, BankQuery, Coin, QuerierWrapper, QueryRequest,
    StdResult, WasmQuery,
};
use cw20::BalanceResponse as Cw20BalanceResponse;

use crate::cw20::is_cw20_token;

pub fn query_balance(
    api: &dyn Api,
    querier: &QuerierWrapper,
    denom: &str,
    address: &Addr,
) -> StdResult<Coin> {
    match is_cw20_token(api, denom) {
        Ok(token_address) => {
            let res =
                querier.query::<Cw20BalanceResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: token_address.to_string(),
                    msg: to_json_binary(&cw20::Cw20QueryMsg::Balance {
                        address: address.to_string(),
                    })?,
                }))?;
            Ok(Coin {
                denom: token_address.to_string(),
                amount: res.balance,
            })
        }
        Err(_) => {
            let balance =
                querier.query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
                    address: address.to_string(),
                    denom: denom.to_string(),
                }))?;
            Ok(balance.amount)
        }
    }
}
