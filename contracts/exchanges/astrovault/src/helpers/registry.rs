use cosmwasm_std::{
    ensure, Deps, QuerierWrapper, StdError, StdResult
};


use astrovault::{
    assets::{asset::AssetInfo, 
        ratio_pools::RatioPoolInfo, 
        pools::PoolInfo as StablePoolInfo, 
        pairs::PairInfo as StandardPoolInfo
    }, 
    ratio_pool_factory::query_msg::QueryMsg as RatioFactoryQueryMsg,
    stable_pool_factory::query_msg::QueryMsg as StableFactoryQueryMsg,
    standard_pool_factory::query_msg::QueryMsg as StandardFactoryQueryMsg,
};


use crate::{
    helpers::pool::query_assets, state::config::get_router_config, types::{config::RouterConfig, pair::PopulatedPair, pool::PoolType}, ContractError
};


fn query_ratio_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<RatioPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioFactoryQueryMsg::Pool { asset_infos }
    )
}

fn query_standard_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<StandardPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardFactoryQueryMsg::Pair { asset_infos }
    )
}

fn query_stable_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: Vec<AssetInfo>
) -> StdResult<StablePoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableFactoryQueryMsg::Pool { asset_infos }
    )
}



pub fn pool_exist_in_registry(
    deps: Deps,
    pair: &PopulatedPair
) -> Result<bool, ContractError> {

    let cfg : RouterConfig = get_router_config(deps.storage)?;

    let pool = pair.pool();
    let pool_type = pool.pool_type;

    let factory_address  = match pool_type {
        PoolType::Ratio => cfg.ratio_pool_factory,
        PoolType::Standard => cfg.standard_pool_factory,
        PoolType::Stable => cfg.stable_pool_factory,
    };

    ensure!(factory_address.is_some(), StdError::GenericErr {
        msg: format!("Factory address not set for pool type: {:?}", pool_type)
    });

    let factory_address = factory_address.as_ref().unwrap();

    let stable_assets = match pool_type {
        PoolType::Stable => query_assets(
                            &deps.querier, 
                            factory_address, 
                            &PoolType::Stable
                        )?
                        .iter()
                        .map(|a| a.info.clone())
                        .collect::<Vec<AssetInfo>>(),

        _ => vec![]
    };

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
            stable_assets
        ).is_ok(),
    };

    Ok(pool_exists)
}
