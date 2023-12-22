use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    helpers::routes::calculate_route,
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
        if pair.route.is_empty() {
            return Err(ContractError::BadRoute {
                msg: "Swap route must not be empty".to_string(),
            });
        }

        let mut deduped_route = pair.route.clone();

        deduped_route.sort();
        deduped_route.dedup();

        if pair.route.len() != deduped_route.len() {
            return Err(ContractError::BadRoute {
                msg: "Swap route must not contain duplicate entries".to_string(),
            });
        }

        for denom in pair.denoms() {
            calculate_route(&deps.querier, &pair, denom.clone()).map_err(|err| {
                ContractError::BadRoute {
                    msg: err.to_string(),
                }
            })?;
        }

        save_pair(deps.storage, &pair)?;
    }

    Ok(Response::new()
        .add_attribute("create_pairs", "true")
        .add_attribute("pairs_created", pairs.len().to_string()))
}

#[cfg(test)]
mod create_pairs_tests {
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        to_json_binary,
    };
    use exchange::msg::ExecuteMsg;

    use crate::{
        contract::execute,
        msg::InternalExternalMsg,
        state::{config::update_config, pairs::find_pair},
        tests::{
            constants::{ADMIN, DENOM_STAKE, DENOM_UOSMO},
            mocks::calc_mock_dependencies,
        },
        types::{config::Config, pair::Pair},
        ContractError,
    };

    #[test]
    fn with_unauthorised_sender_fails() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let info_with_unauthorised_sender = mock_info("not-admin", &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info_with_unauthorised_sender,
            ExecuteMsg::InternalMsg {
                msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                    pairs: vec![Pair {
                        base_denom: String::from("base"),
                        quote_denom: String::from("quote"),
                        route: vec![0],
                    }],
                })
                .unwrap(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {})
    }

    #[test]
    fn with_empty_route_fails() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::InternalMsg {
                msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                    pairs: vec![Pair {
                        quote_denom: DENOM_UOSMO.to_string(),
                        base_denom: DENOM_STAKE.to_string(),
                        route: vec![],
                    }],
                })
                .unwrap(),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::BadRoute {
                msg: "Swap route must not be empty".to_string(),
            }
        )
    }

    #[test]
    fn with_invalid_route_fails() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::InternalMsg {
                msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                    pairs: vec![Pair {
                        quote_denom: DENOM_UOSMO.to_string(),
                        base_denom: DENOM_STAKE.to_string(),
                        route: vec![2],
                    }],
                })
                .unwrap(),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::BadRoute {
                msg: "Generic error: denom ustake not found in pool id 2".to_string(),
            }
        )
    }

    #[test]
    fn with_duplicate_route_entries_fails() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let create_pair_execute_message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    base_denom: DENOM_UOSMO.to_string(),
                    quote_denom: DENOM_STAKE.to_string(),
                    route: vec![4, 1, 4, 1],
                }],
            })
            .unwrap(),
        };

        let err = execute(deps.as_mut(), env, info, create_pair_execute_message).unwrap_err();

        assert_eq!(
            err,
            ContractError::BadRoute {
                msg: "Swap route must not contain duplicate entries".to_string()
            }
        )
    }

    #[test]
    fn with_valid_id_should_succeed() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let create_pair_execute_message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    base_denom: DENOM_UOSMO.to_string(),
                    quote_denom: DENOM_STAKE.to_string(),
                    route: vec![3],
                }],
            })
            .unwrap(),
        };

        execute(deps.as_mut(), env, info, create_pair_execute_message).unwrap();

        let pair = find_pair(
            deps.as_ref().storage,
            [DENOM_UOSMO.to_string(), DENOM_STAKE.to_string()],
        )
        .unwrap();

        assert_eq!(pair.base_denom, DENOM_UOSMO.to_string());
        assert_eq!(pair.quote_denom, DENOM_STAKE.to_string());
        assert_eq!(pair.route, vec![3]);
    }

    #[test]
    fn that_already_exists_should_update_it() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let original_message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    base_denom: DENOM_UOSMO.to_string(),
                    quote_denom: DENOM_STAKE.to_string(),
                    route: vec![4, 1],
                }],
            })
            .unwrap(),
        };

        let message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    base_denom: DENOM_UOSMO.to_string(),
                    quote_denom: DENOM_STAKE.to_string(),
                    route: vec![3],
                }],
            })
            .unwrap(),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), original_message).unwrap();

        let denoms = [DENOM_UOSMO.to_string(), DENOM_STAKE.to_string()];

        let original_pair = find_pair(deps.as_ref().storage, denoms.clone()).unwrap();

        execute(deps.as_mut(), env, info, message).unwrap();

        let pair = find_pair(deps.as_ref().storage, denoms).unwrap();

        assert_eq!(original_pair.route, vec![4, 1]);
        assert_eq!(pair.route, vec![3]);
    }

    #[test]
    fn with_switched_denoms_should_overwrite_it() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        update_config(deps.as_mut().storage, Config::default()).unwrap();

        let original_message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    quote_denom: DENOM_UOSMO.to_string(),
                    base_denom: DENOM_STAKE.to_string(),
                    route: vec![1, 4],
                }],
            })
            .unwrap(),
        };

        let message = ExecuteMsg::InternalMsg {
            msg: to_json_binary(&InternalExternalMsg::CreatePairs {
                pairs: vec![Pair {
                    quote_denom: DENOM_UOSMO.to_string(),
                    base_denom: DENOM_STAKE.to_string(),
                    route: vec![3],
                }],
            })
            .unwrap(),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), original_message).unwrap();

        let denoms = [DENOM_UOSMO.to_string(), DENOM_STAKE.to_string()];

        let original_pair = find_pair(deps.as_ref().storage, denoms.clone()).unwrap();

        execute(deps.as_mut(), env, info, message).unwrap();

        let pair = find_pair(deps.as_ref().storage, denoms).unwrap();

        assert_eq!(original_pair.route, vec![1, 4]);
        assert_eq!(pair.route, vec![3]);
    }
}
