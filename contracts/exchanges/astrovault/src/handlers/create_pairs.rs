use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    helpers::validated::validated_pair_on_creation,
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

    let pairs_len = pairs.len();

    for pair in pairs {
        let validated = validated_pair_on_creation(deps.as_ref(), &pair)?;
        save_pair(deps.storage, &validated)?;
    }

    Ok(Response::new()
        .add_attribute("create_pairs", "true")
        .add_attribute("pairs_created", pairs_len.to_string()))
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
        types::pair::{Pair, PairType, PopulatedPair},
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
                router_address: Addr::unchecked("router-address"),
                allow_implicit: None,
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
                router_address: Addr::unchecked("router-address"),
                allow_implicit: None,
            },
        )
        .unwrap();

        let pair = PopulatedPair::default();
        let pool = pair.pool();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let new_address = String::from("new-pair-address");

        create_pairs_handler(
            deps.as_mut(),
            mock_info(ADMIN, &[]),
            vec![Pair {
                pair_type: PairType::Direct {
                    address: new_address.clone(),
                    pool_type: pool.pool_type,
                },
                base_asset: pool.base_asset.clone(),
                quote_asset: pool.quote_asset.clone(),
            }],
        )
        .unwrap();

        let updated_pair = find_pair(deps.as_ref().storage, pair.denoms()).unwrap();

        let updated_pool = updated_pair.pool();

        assert_ne!(pool.address, updated_pool.address);
        assert_eq!(updated_pool.address, new_address);
    }
}
