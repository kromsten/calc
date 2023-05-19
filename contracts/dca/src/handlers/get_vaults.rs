use crate::state::old_vaults::get_vaults;
use crate::{helpers::validation_helpers::assert_page_limit_is_valid, msg::VaultsResponse};
use cosmwasm_std::{Deps, Env, StdResult, Uint128};

pub fn get_vaults_handler(
    deps: Deps,
    env: Env,
    start_after: Option<Uint128>,
    limit: Option<u16>,
) -> StdResult<VaultsResponse> {
    assert_page_limit_is_valid(deps.storage, limit)?;

    let vaults = get_vaults(deps.storage, env, start_after, limit)?;

    Ok(VaultsResponse { vaults })
}
