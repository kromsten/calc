#![allow(unused_variables, unused_imports)]

use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    helpers::{
        pool::validated_direct_pair, 
        route::{validated_route_pairs, validated_route_pairs_to_save}}, 
        state::{config::get_config, pairs::{save_pair, save_route_pair}}, 
        types::pair::Pair, ContractError
};

use astrovault::router::handle_msg;
use astrovault::router::query_msg;
use astrovault::router::state::Hop;


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
        if pair.is_pool_pair() {
            let pair = validated_direct_pair(deps.as_ref(), &pair)?;
            save_pair(deps.storage, &pair)?;
        } else {
            let route_pairs = validated_route_pairs_to_save(deps.as_ref(), &pair)?;
            for route_pair in route_pairs {
                save_pair(deps.storage, &route_pair)?;
            }
            save_route_pair(deps.storage, &pair)?;
        }
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
        types::pair::{Pair, PairType},
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
            },
        )
        .unwrap();

        let pair = Pair::default();
        let pool = pair.pool_info();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let new_address = String::from("new-pair-address");

        create_pairs_handler(
            deps.as_mut(),
            mock_info(ADMIN, &[]),
            vec![Pair {
                pair_type: PairType::Direct { 
                    address: new_address.clone(), 
                    base_index: None,
                    quote_index: None,
                    pool_type: pool.pool_type
                },
                ..pair.clone()
            }],
        )
        .unwrap();

        let updated_pair = find_pair(deps.as_ref().storage, pair.denoms()).unwrap();

        let updated_pool = updated_pair.pool_info();

        assert_ne!(pool.address, updated_pool.address);
        assert_eq!(updated_pool.address, new_address);
    }
}
