use astrovault::assets::asset::{AssetInfo, Asset};
use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult, QueryRequest, WasmQuery, to_json_binary, Uint128, BalanceResponse, BankQuery};
use cw20::BalanceResponse as Cw20BalanceResponse;



pub fn get_asset_balance(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
    address: Addr,
) -> StdResult<Uint128> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            let res : Cw20BalanceResponse =  querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20QueryMsg::Balance { 
                    address: address.to_string()
                })?,
            }))?;
            Ok(res.balance)
        }
        AssetInfo::NativeToken { denom, .. } => {
            let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
                address: address.to_string(),
                denom,
            }))?;
            Ok(balance.amount.amount)
        }
    }
}


pub fn to_asset_info(denom: impl Into<String>) -> AssetInfo {
    let denom = denom.into();
    if denom.starts_with("archway")  ||
        // or length is bigger than 10 characters and it isn't ibc denom'
        (denom.len() > 10 && !denom.starts_with("ibc/")
    ) {
        AssetInfo::Token { contract_addr: denom }
    } else {
        AssetInfo::NativeToken { denom }
    }
}

/// Helper function that detect if Coin type is possibly a wrapper for 
/// cw20 based token serving as unifing interface
pub fn coin_to_asset(coin: Coin) -> Asset {
    Asset { 
        info: to_asset_info(coin.denom), 
        amount: coin.amount 
    }
}


pub fn asset_to_coin(asset: Asset) -> Coin {
    Coin { denom: asset.info.to_string(), amount: asset.amount }
}

