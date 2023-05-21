use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Storage};
use cw_storage_plus::Item;

use crate::types::fee_collector::FeeCollector;

#[cw_serde]
pub struct OldConfig {
    pub admin: Addr,
    pub executors: Vec<Addr>,
    pub fee_collectors: Vec<FeeCollector>,
    pub default_swap_fee_percent: Decimal,
    pub delegation_fee_percent: Decimal,
    pub staking_router_address: Addr,
    pub default_page_limit: u16,
    pub paused: bool,
    pub risk_weighted_average_escrow_level: Decimal,
}

const CONFIG: Item<OldConfig> = Item::new("config_v7");

pub fn get_old_config(store: &dyn Storage) -> StdResult<OldConfig> {
    CONFIG.load(store)
}

pub fn update_old_config(store: &mut dyn Storage, config: OldConfig) -> StdResult<OldConfig> {
    if config.default_swap_fee_percent > Decimal::percent(100) {
        return Err(StdError::generic_err(
            "swap_fee_percent must be less than 100%, and expressed as a ratio out of 1 (i.e. use 0.015 to represent a fee of 1.5%)",
        ));
    }

    if config.delegation_fee_percent > Decimal::percent(100) {
        return Err(StdError::generic_err(
            "delegation_fee_percent must be less than 100%, and expressed as a ratio out of 1 (i.e. use 0.015 to represent a fee of 1.5%)",
        ));
    }

    CONFIG.save(store, &config)?;
    Ok(config)
}
