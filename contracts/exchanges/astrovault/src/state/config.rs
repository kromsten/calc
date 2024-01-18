use crate::types::config::{Config, RouterConfig};
use cosmwasm_std::{QuerierWrapper, StdResult, Storage};
use cw_storage_plus::Item;
use astrovault::router::query_msg as RouterQuery;

const CONFIG: Item<Config> = Item::new("config_v2");
const ROUTER_CONFIG: Item<RouterConfig> = Item::new("rc_v2");


pub fn get_config(store: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(store)
}

pub fn update_config(store: &mut dyn Storage, config: Config) -> StdResult<Config> {
    CONFIG.save(store, &config)?;
    Ok(config)
}


pub fn update_router_config(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    router: &str,
) -> StdResult<()> {
    let res = querier.query_wasm_smart::<RouterQuery::ConfigResponse>(
        router, 
        &RouterQuery::QueryMsg::Config {}
    )?;

    ROUTER_CONFIG.save(storage, &res)?;
    Ok(())
}