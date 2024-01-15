use cosmwasm_std::{Event, StdError, StdResult, CosmosMsg, to_json_binary, WasmMsg, BankMsg, Coin, Uint128, Binary, Decimal, QuerierWrapper, Addr};
use cw20::Cw20ExecuteMsg;

use astrovault::{
    standard_pool::{
        handle_msg::ExecuteMsg as StandardExecute,
        query_msg::{
            QueryMsg as StandardQuery,
            PoolResponse as StandardPoolResponse,
            SimulationResponse
        }
    },
    ratio_pool::{
        handle_msg::ExecuteMsg as RatioExecute,
        query_msg::{
            QueryMsg as RatioQuery,
            PoolResponse as RatioPoolResponse,
        },
    },
    stable_pool::{
        handle_msg::ExecuteMsg as StableExecute, 
        query_msg::{
            QueryMsg as StableQuery, 
            PoolResponse as StablePoolResponse,
            StablePoolQuerySwapSimulation, 
        }
    },
    assets::asset::{Asset, AssetInfo}, 
    ratio_pool_factory::query_msg::SwapCalcResponse
};

use crate::types::pair::PoolType;

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


fn swap_standard_msg(
    offer_asset: Asset,
    expected_return: Option<Uint128>,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    to: Option<String>,
) -> StdResult<Binary> {
    let msg = StandardExecute::Swap {
        offer_asset,
        belief_price,
        max_spread,
        expected_return,
        to,
    };

    to_json_binary(&msg)
}


fn swap_stable_msg(
    expected_return: Option<Uint128>,
    swap_to_asset_index: Option<u32>,
    to: Option<String>,
) -> StdResult<Binary> {    
    let msg = StableExecute::Swap {
        expected_return,
        to,
        swap_to_asset_index: swap_to_asset_index.unwrap_or(1),
    };
    to_json_binary(&msg)
}


pub fn swap_ratio_msg(
    expected_return: Option<Uint128>,
    to: Option<String>,
) -> StdResult<Binary> {
    let msg = RatioExecute::Swap {
        expected_return,
        to,
    };
    to_json_binary(&msg)
}




fn simulate_swap_standard(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    offer_asset: Asset,
) -> StdResult<Uint128> {
    let res = querier.query_wasm_smart::<SimulationResponse>(
        contract_addr, 
        &StandardQuery::Simulation { offer_asset }
    )?;
    Ok(res.return_amount)
}


fn simulate_swap_stable(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    offer_asset: Asset,
    target_asset: AssetInfo
) -> StdResult<Uint128> {

    let assets = query_assets(querier, contract_addr.clone(), PoolType::Stable)?;

    let from_index = assets
        .iter()
        .position(|a| a.info.equal(&offer_asset.info))
        .unwrap_or(0) as u32;

    let to_index = assets
        .iter()
        .position(|a| a.info.equal(&target_asset))
        .unwrap_or(1);

    let msg = StableQuery::SwapSimulation { 
        amount: offer_asset.amount, 
        swap_from_asset_index: from_index, 
        swap_to_asset_index: to_index as u32
    };

    let res = querier.query_wasm_smart::<StablePoolQuerySwapSimulation>(
        contract_addr, 
        &msg
    )?;

    let swap_amount = res.swap_to_assets_amount.get(to_index).unwrap().clone();
    let mint_amount = res.mint_to_assets_amount.get(to_index).unwrap().clone();

    Ok(swap_amount.checked_add(mint_amount)?)
}


fn simulate_swap_ratio(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    offer_asset: Asset,
) -> StdResult<Uint128> {

    let swap_from_asset_index = query_asset_index(
        querier, 
        contract_addr.clone(), 
        PoolType::Ratio, 
        offer_asset.info
    )? as u8;

    let msg = RatioQuery::SwapSimulation { 
        amount: offer_asset.amount, 
        swap_from_asset_index: swap_from_asset_index, 
    };

    let res = querier.query_wasm_smart::<SwapCalcResponse>(
        contract_addr, 
        &msg
    )?;

    Ok(res.to_amount_minus_fee)
}



pub fn pool_swap_simulate(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    pool_type: PoolType,
    offer_asset: Asset,
    target_asset: AssetInfo,
) -> StdResult<Uint128> {
    match pool_type {
        PoolType::Standard => simulate_swap_standard(querier, contract_addr, offer_asset),
        PoolType::Stable => simulate_swap_stable(querier, contract_addr, offer_asset, target_asset),
        PoolType::Ratio => simulate_swap_ratio(querier, contract_addr, offer_asset),
    }
}


pub fn pool_swap_binary_msg(
    pool_type: PoolType,
    offer_asset: Asset,
    expected_return: Option<Uint128>,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    swap_to_asset_index: Option<u32>,
    to: Option<String>,
) -> StdResult<Binary> {
    match pool_type {
        PoolType::Standard => swap_standard_msg(offer_asset, expected_return, belief_price, max_spread, to),
        PoolType::Stable => swap_stable_msg(expected_return, swap_to_asset_index, to),
        PoolType::Ratio => swap_ratio_msg(expected_return, to),
    }
}


pub fn ratio_pool_response(
    querier: &QuerierWrapper,
    contract_addr: Addr,
) -> StdResult<RatioPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioQuery::Pool {}
    )
}


pub fn stable_pool_response(
    querier: &QuerierWrapper,
    contract_addr: Addr,
) -> StdResult<StablePoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableQuery::Pool {}
    )
}

pub fn standard_pool_response(
    querier: &QuerierWrapper,
    contract_addr: Addr,
) -> StdResult<StandardPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardQuery::Pool {}
    )
}


pub fn query_assets(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    pool_type: PoolType,
) -> StdResult<Vec<Asset>> {
    let assets = match pool_type {
        PoolType::Stable => stable_pool_response(querier, contract_addr)?.assets,
        PoolType::Ratio => ratio_pool_response(querier, contract_addr)?.assets.into(),
        PoolType::Standard => standard_pool_response(querier, contract_addr)?.assets.into(),
    };
    
    Ok(assets)
}


pub fn query_asset_index(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    pool_type: PoolType,
    asset_info: AssetInfo,
) -> StdResult<u32> {
    let assets = query_assets(querier, contract_addr, pool_type)?;
    
    Ok(assets
        .iter()
        .position(|a| a.info == asset_info)
        .unwrap_or(0) as u32)
}