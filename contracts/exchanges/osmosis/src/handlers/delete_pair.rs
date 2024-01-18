use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    state::{
        config::get_config,
        pairs::{delete_pair, find_pair},
    },
    types::pair::Pair,
    ContractError,
};

pub fn delete_pairs_handler(
    deps: DepsMut,
    info: MessageInfo,
    pairs: Vec<Pair>,
) -> Result<Response, ContractError> {
    let config = get_config(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for pair in pairs.clone() {
        let stored_pair = find_pair(deps.storage, pair.denoms());

        if let Ok(pair) = stored_pair {
            delete_pair(deps.storage, &pair);
        }
    }

    Ok(Response::new()
        .add_attribute("delete_pairs", "true")
        .add_attribute("pairs_deleted", pairs.len().to_string()))
}
