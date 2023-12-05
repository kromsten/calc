use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    state::{config::get_config, pairs::save_pair},
    types::pair::Pair,
    ContractError,
};

pub fn create_pairs_handler(
    deps: DepsMut,
    info: MessageInfo,
    pairs: Vec<Pair>,
) -> Result<Response, ContractError> {
    let config = get_config(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for pair in pairs.clone() {
        deps.api.addr_validate(pair.address.as_ref())?;
        save_pair(deps.storage, &pair)?;
    }

    Ok(Response::new()
        .add_attribute("create_pairs", "true")
        .add_attribute("pairs_created", pairs.len().to_string()))
}

#[cfg(test)]
mod create_pairs_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::{
        contract::instantiate,
        handlers::create_pairs::create_pairs_handler,
        msg::InstantiateMsg,
        state::pairs::{find_pair, save_pair},
        tests::constants::ADMIN,
        types::pair::Pair,
        ContractError,
    };

    #[test]
    fn with_non_admin_sender_fails() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            InstantiateMsg {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked("dca-contract-address"),
            },
        )
        .unwrap();

        assert_eq!(
            create_pairs_handler(deps.as_mut(), mock_info("not-admin", &[]), vec![]).unwrap_err(),
            ContractError::Unauthorized {}
        )
    }

    #[test]
    fn overwrites_existing_pair() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            InstantiateMsg {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked("dca-contract-address"),
            },
        )
        .unwrap();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let new_address = Addr::unchecked("new-pair-address");

        create_pairs_handler(
            deps.as_mut(),
            mock_info(ADMIN, &[]),
            vec![Pair {
                address: new_address.clone(),
                ..pair.clone()
            }],
        )
        .unwrap();

        let updated_pair = find_pair(deps.as_ref().storage, pair.denoms()).unwrap();

        assert_ne!(pair.address, updated_pair.address);
        assert_eq!(updated_pair.address, new_address);
    }
}
