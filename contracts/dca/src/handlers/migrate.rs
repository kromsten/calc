use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    msg::MigrateMsg,
    state::config::update_config,
    types::config::Config,
};
use cosmwasm_std::{DepsMut, Response, StdError};
use cw2::{get_contract_version, set_contract_version};

pub fn migrate_handler(deps: DepsMut, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    if contract_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }

    #[allow(clippy::cmp_owned)]
    if contract_version.version > CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    update_config(
        deps.storage,
        Config {
            exchange_contract_address: msg.exchange_contract_address.clone(),
            admin: msg.admin.clone(),
            executors: msg.executors.clone(),
            fee_collectors: msg.fee_collectors.clone(),
            default_swap_fee_percent: msg.default_swap_fee_percent,
            weighted_scale_swap_fee_percent: msg.weighted_scale_swap_fee_percent,
            automation_fee_percent: msg.automation_fee_percent,
            default_page_limit: msg.default_page_limit,
            paused: msg.paused,
            risk_weighted_average_escrow_level: msg.risk_weighted_average_escrow_level,
            twap_period: msg.twap_period,
            default_slippage_tolerance: msg.default_slippage_tolerance,
            old_staking_router_address: msg.old_staking_router_address.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("migrate", "true")
        .add_attribute("msg", format!("{:?}", msg)))
}
