use crate::types::config::{Config, RouterConfig};
use cosmwasm_std::{QuerierWrapper, StdResult, Storage};
use cw_storage_plus::Item;

#[cfg(target_arch = "wasm32")]
use astrovault::router::query_msg as RouterQuery;

const CONFIG            : Item<Config>          = Item::new("config_v2");
const ROUTER_CONFIG     : Item<RouterConfig>    = Item::new("rc_v2");


pub fn get_config(store: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(store)
}

pub fn update_config(store: &mut dyn Storage, config: Config) -> StdResult<Config> {
    CONFIG.save(store, &config)?;
    Ok(config)
}


pub fn get_router_config(
    storage: &dyn Storage,
) -> StdResult<RouterConfig> {
    ROUTER_CONFIG.load(storage)
}


#[cfg(target_arch = "wasm32")]
pub fn update_router_config(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    router: &str,
) -> StdResult<()> {
    use cosmwasm_std::from_json;

    let res = querier.query_wasm_smart::<RouterQuery::ConfigResponse>(
        router, 
        &RouterQuery::QueryMsg::Config {}
    )?;

    ROUTER_CONFIG.save(storage, &res)?;
    Ok(())
}


#[cfg(not(target_arch = "wasm32"))]
pub fn update_router_config(
    _: &QuerierWrapper,
    storage: &mut dyn Storage,
    owner: &str,
) -> StdResult<()> {
    ROUTER_CONFIG.save(storage, &RouterConfig {
        owner: owner.to_string(),
        cashback: None,
        standard_pool_factory: None,
        stable_pool_factory: None,
        ratio_pool_factory: None,
    })
}