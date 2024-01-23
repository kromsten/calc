use cosmwasm_std::{ensure, to_json_binary, Binary, Decimal, Deps, QuerierWrapper, StdError, StdResult, Storage, Uint128};

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
    assets::{asset::{Asset, AssetInfo}, 
        ratio_pools::RatioPoolInfo, 
        pools::PoolInfo as StablePoolInfo, 
        pairs::PairInfo as StandardPoolInfo
    }, 
    ratio_pool_factory::query_msg::{SwapCalcResponse, QueryMsg as RatioFactoryQueryMsg},
    stable_pool_factory::query_msg::QueryMsg as StableFactoryQueryMsg,
    standard_pool_factory::query_msg::QueryMsg as StandardFactoryQueryMsg,
};

use crate::{state::config::get_router_config, types::{config::RouterConfig, pair::{Pair, PoolType}}};

#[cfg(not(target_arch = "wasm32"))]
pub fn swap_msg(
    _: &QuerierWrapper,
    _: &str,
    pool_type: &PoolType,
    offer_asset: Asset,
    _: Asset,
) -> StdResult<Binary> {
    pool_swap_binary_msg(
        pool_type,
        offer_asset,
        None,
        None,
        None,
        None,
        None
    )
}


#[cfg(target_arch = "wasm32")]
pub fn swap_msg(
    querier: &QuerierWrapper,
    address: &str,
    pool_type: &PoolType,
    offer_asset: Asset,
    min_amount: Asset,
) -> StdResult<Binary> {

    let swap_to_asset_index = match pool_type {
        PoolType::Ratio => None,
        _ => {
            let assets = query_assets(
                    querier, 
                    address,
                    pool_type
            )?;
            Some(
                assets
                .iter()
                .position(|a| a.info.equal(&min_amount.info))
                .unwrap_or(1) as u32
            )
        }
    };

    pool_swap_binary_msg(
        pool_type,
        offer_asset,
        None,
        None,
        None,
        swap_to_asset_index,
        None
    )
}


pub fn pool_exist(
    querier: &QuerierWrapper,
    _storage: &dyn Storage,
    pool_address: &str,
    pool_type: &PoolType,
) -> StdResult<()> {
    // let config = get_router_config(storage)?;

    if query_assets(querier, pool_address, pool_type).is_err() {

    };
    

    Ok(())
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
    contract_addr: &str,
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
    contract_addr: &str,
    offer_asset: Asset,
    target_asset: &AssetInfo
) -> StdResult<Uint128> {

    let assets = query_assets(querier, contract_addr, &PoolType::Stable)?;

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
    contract_addr: &str,
    offer_asset: Asset,
) -> StdResult<Uint128> {

    let swap_from_asset_index = query_asset_index(
        querier, 
        contract_addr, 
        &PoolType::Ratio, 
        &offer_asset.info
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
    contract_addr: &str,
    pool_type: &PoolType,
    offer_asset: Asset,
    target_asset: AssetInfo,
) -> StdResult<Uint128> {
    match pool_type {
        PoolType::Standard => simulate_swap_standard(querier, contract_addr, offer_asset),
        PoolType::Stable => simulate_swap_stable(querier, contract_addr, offer_asset, &target_asset),
        PoolType::Ratio => simulate_swap_ratio(querier, contract_addr, offer_asset),
    }
}


pub fn pool_swap_binary_msg(
    pool_type: &PoolType,
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
    contract_addr: &str,
) -> StdResult<RatioPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioQuery::Pool {}
    )
}


pub fn stable_pool_response(
    querier: &QuerierWrapper,
    contract_addr: &str,
) -> StdResult<StablePoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableQuery::Pool {}
    )
}

pub fn standard_pool_response(
    querier: &QuerierWrapper,
    contract_addr: &str,
) -> StdResult<StandardPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardQuery::Pool {}
    )
}


pub fn query_assets(
    querier: &QuerierWrapper,
    contract_addr: &str,
    pool_type: &PoolType,
) -> StdResult<Vec<Asset>> {
    let assets = match pool_type {
        PoolType::Stable => stable_pool_response(querier, contract_addr)?.assets,
        PoolType::Ratio => ratio_pool_response(querier, contract_addr)?.assets.into(),
        PoolType::Standard => standard_pool_response(querier, contract_addr)?.assets.into(),
    };
    Ok(assets)
}



pub fn query_ratio_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<RatioPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioFactoryQueryMsg::Pool { asset_infos }
    )
}

pub fn query_standard_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<StandardPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardFactoryQueryMsg::Pair { asset_infos }
    )
}

pub fn query_stable_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: Vec<AssetInfo>
) -> StdResult<StablePoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableFactoryQueryMsg::Pool { asset_infos }
    )
}



pub fn query_pool_exist(
    deps: Deps,
    pair: &Pair
) -> StdResult<bool> {

    let cfg : RouterConfig = get_router_config(deps.storage)?;
    
    let pool_type = pair.pool_type.clone().unwrap();

    let factory_address  = match pool_type {
        PoolType::Ratio => cfg.ratio_pool_factory,
        PoolType::Standard => cfg.standard_pool_factory,
        PoolType::Stable => cfg.stable_pool_factory,
    };

    ensure!(factory_address.is_some(), StdError::GenericErr {
        msg: format!("Factory address not set for pool type: {:?}", pool_type)
    });

    let factory_address = factory_address.as_ref().unwrap();


    let pool_exists = match pool_type {
        PoolType::Ratio => query_ratio_pool_info(
            &deps.querier, 
            factory_address, 
            pair.assets()
        ).is_ok(),
        PoolType::Standard => query_standard_pool_info(
            &deps.querier, 
            factory_address, 
            pair.assets()
        ).is_ok(),
        PoolType::Stable => query_stable_pool_info(
            &deps.querier, 
            factory_address, 
            pair.assets().into()
        ).is_ok(),
    };

    Ok(pool_exists)
}


pub fn query_asset_index(
    querier: &QuerierWrapper,
    contract_addr: &str,
    pool_type: &PoolType,
    asset_info: &AssetInfo,
) -> StdResult<u32> {
    let assets = query_assets(querier, contract_addr, pool_type)?;
    
    Ok(assets
        .iter()
        .position(|a| a.info == *asset_info)
        .unwrap_or(0) as u32)
}