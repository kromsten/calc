use crate::state::old_vaults::get_old_vaults_by_address as fetch_vaults_by_address;
use crate::{helpers::validation::assert_page_limit_is_valid, msg::VaultsResponse};
use base::vaults::vault::OldVaultStatus;
use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

pub fn get_vaults_by_address(
    deps: Deps,
    address: Addr,
    status: Option<OldVaultStatus>,
    start_after: Option<Uint128>,
    limit: Option<u16>,
) -> StdResult<VaultsResponse> {
    deps.api.addr_validate(&address.to_string())?;
    assert_page_limit_is_valid(limit)?;

    let vaults = fetch_vaults_by_address(deps.storage, address, status, start_after, limit)?;

    Ok(VaultsResponse { vaults })
}
