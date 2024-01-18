use crate::types::{pair::Pair, position_type::PositionType};
use cosmwasm_std::{from_json, QuerierWrapper, StdError, StdResult};
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool as ConcentratedLiquidityPool;
use osmosis_std::types::osmosis::cosmwasmpool::v1beta1::{CosmWasmPool, InstantiateMsg};
use osmosis_std::types::osmosis::gamm::poolmodels::stableswap::v1beta1::Pool as StableSwapPool;
use osmosis_std::types::osmosis::gamm::v1beta1::Pool as GammPool;
use osmosis_std::types::osmosis::poolmanager::v1beta1::{PoolmanagerQuerier, SwapAmountInRoute};
use prost::DecodeError;

pub fn get_token_out_denom(
    querier: &QuerierWrapper,
    token_in_denom: String,
    pool_id: u64,
    next_pool_id: u64,
) -> StdResult<String> {
    let pool_assets = get_pool_assets(querier, pool_id)?;

    if !pool_assets.contains(&token_in_denom) {
        return Err(StdError::generic_err(format!(
            "denom {} not found in pool id {}",
            token_in_denom, pool_id
        )));
    }

    let next_pool_assets = get_pool_assets(querier, next_pool_id)?;

    let intersecting_assets = pool_assets
        .iter()
        .filter(|asset| next_pool_assets.contains(*asset))
        .collect::<Vec<&String>>();

    if intersecting_assets.is_empty() {
        return Err(StdError::generic_err(format!(
            "pool {} contains no assets of the pool {}",
            next_pool_id, pool_id
        )));
    }

    Ok(intersecting_assets[0].clone())
}

pub fn get_pool_assets(querier: &QuerierWrapper, pool_id: u64) -> Result<Vec<String>, StdError> {
    PoolmanagerQuerier::new(querier).pool(pool_id)?.pool.map_or(
        Err(StdError::generic_err("pool not found")),
        |pool| match pool.type_url.as_str() {
            GammPool::TYPE_URL => pool
                .try_into()
                .map(|pool: GammPool| {
                    pool.pool_assets
                        .into_iter()
                        .map(|asset| asset.token.unwrap().denom)
                        .collect::<Vec<String>>()
                })
                .map_err(|e: DecodeError| StdError::ParseErr {
                    target_type: GammPool::TYPE_URL.to_string(),
                    msg: e.to_string(),
                }),
            ConcentratedLiquidityPool::TYPE_URL => pool
                .try_into()
                .map(|pool: ConcentratedLiquidityPool| vec![pool.token0, pool.token1])
                .map_err(|e: DecodeError| StdError::ParseErr {
                    target_type: ConcentratedLiquidityPool::TYPE_URL.to_string(),
                    msg: e.to_string(),
                }),
            StableSwapPool::TYPE_URL => pool
                .try_into()
                .map(|pool: StableSwapPool| {
                    pool.pool_liquidity
                        .into_iter()
                        .map(|asset| asset.denom)
                        .collect::<Vec<String>>()
                })
                .map_err(|e: DecodeError| StdError::ParseErr {
                    target_type: StableSwapPool::TYPE_URL.to_string(),
                    msg: e.to_string(),
                }),
            CosmWasmPool::TYPE_URL => pool
                .try_into()
                .map(|pool: CosmWasmPool| {
                    from_json(&pool.instantiate_msg)
                        .map(|msg: InstantiateMsg| {
                            msg.pool_asset_denoms.into_iter().collect::<Vec<String>>()
                        })
                        .expect(&format!(
                            "pool assets for cosmwasm pool id: {}",
                            pool.pool_id
                        ))
                })
                .map_err(|e: DecodeError| StdError::ParseErr {
                    target_type: CosmWasmPool::TYPE_URL.to_string(),
                    msg: e.to_string(),
                }),
            _ => Err(StdError::generic_err(format!(
                "pool type {} not supported",
                pool.type_url
            ))),
        },
    )
}

pub fn calculate_route(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_denom: String,
) -> StdResult<Vec<SwapAmountInRoute>> {
    let pair_denoms = pair.denoms();
    let target_denom = pair.other_denom(swap_denom.clone());

    if !pair_denoms.contains(&swap_denom) {
        return Err(StdError::generic_err(format!(
            "swap denom {} not in pair denoms {:?}",
            swap_denom, pair_denoms
        )));
    }

    let pool_ids = match pair.position_type(swap_denom.clone()) {
        PositionType::Enter => pair.route.clone(),
        PositionType::Exit => pair.route.clone().into_iter().rev().collect(),
    };

    let initial_pool_id = pool_ids.first().unwrap();
    let initial_pool_assets = get_pool_assets(querier, *initial_pool_id)?;

    if !initial_pool_assets.contains(&swap_denom) {
        return Err(StdError::generic_err(format!(
            "denom {} not found in pool id {}",
            swap_denom, initial_pool_id
        )));
    }

    let mut route: Vec<SwapAmountInRoute> = vec![];
    let mut token_in_denom = swap_denom;

    for adjacent_pools in pool_ids.windows(2).into_iter() {
        let token_out_denom = get_token_out_denom(
            querier,
            token_in_denom.clone(),
            adjacent_pools[0],
            adjacent_pools[1],
        )?;

        route.push(SwapAmountInRoute {
            pool_id: adjacent_pools[0],
            token_out_denom: token_out_denom.clone(),
        });

        token_in_denom = token_out_denom;
    }

    let final_pool_id = *pool_ids.last().unwrap();
    let final_pool_assets = get_pool_assets(querier, final_pool_id)?;

    if !final_pool_assets.contains(&target_denom) {
        return Err(StdError::generic_err(format!(
            "pool denoms {:?} do not contain target denom {}",
            pair_denoms, target_denom,
        )));
    }

    route.push(SwapAmountInRoute {
        pool_id: final_pool_id,
        token_out_denom: target_denom,
    });

    Ok(route)
}

#[cfg(test)]
mod get_token_out_denom_tests {
    use super::get_token_out_denom;
    use crate::{
        tests::{
            constants::{DENOM_UATOM, DENOM_UOSMO},
            mocks::calc_mock_dependencies,
        },
        types::pair::Pair,
    };

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0, 1],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let swap_denom = "not_in_pair".to_string();

        let err = get_token_out_denom(
            &deps.as_ref().querier,
            swap_denom.clone(),
            pair.route[0],
            pair.route[1],
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: denom {} not found in pool id {}",
                swap_denom, pair.route[0]
            )
        );
    }
}

#[cfg(test)]
mod calculate_route_tests {
    use super::calculate_route;
    use crate::{
        tests::{
            constants::{DENOM_UATOM, DENOM_UION, DENOM_UOSMO, DENOM_USDC},
            mocks::calc_mock_dependencies,
        },
        types::pair::Pair,
    };
    use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountInRoute;

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let swap_denom = "not_in_pair".to_string();

        let err = calculate_route(&deps.as_ref().querier, &pair, swap_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: swap denom {} not in pair denoms {:?}",
                swap_denom,
                pair.denoms()
            )
        );
    }

    #[test]
    fn when_initial_pool_does_not_contain_swap_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![2],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: denom {} not found in pool id {}",
                pair.quote_denom, pair.route[0]
            )
        );
    }

    #[test]
    fn when_intermediary_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0, 2],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool {} contains no assets of the pool {}",
                pair.route[1], pair.route[0]
            )
        );
    }

    #[test]
    fn when_final_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0, 1],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_USDC.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool denoms {:?} do not contain target denom {}",
                pair.denoms(),
                pair.base_denom
            )
        );
    }

    #[test]
    fn calculates_1_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 0,
                token_out_denom: DENOM_UOSMO.to_string(),
            }]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UOSMO.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 0,
                token_out_denom: DENOM_UATOM.to_string(),
            }]
        );
    }

    #[test]
    fn calculates_2_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![0, 1],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UION.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 0,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 1,
                    token_out_denom: DENOM_UION.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UION.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 1,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 0,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );
    }

    #[test]
    fn calculates_3_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![2, 1, 0],
            quote_denom: DENOM_USDC.to_string(),
            base_denom: DENOM_UATOM.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_USDC.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 2,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 1,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 0,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 0,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 1,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 2,
                    token_out_denom: DENOM_USDC.to_string(),
                },
            ]
        );
    }

    #[test]
    fn with_cl_pools_when_swap_denom_not_in_pair_denoms_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![5],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let swap_denom = "not_in_pair".to_string();

        let err = calculate_route(&deps.as_ref().querier, &pair, swap_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: swap denom {} not in pair denoms {:?}",
                swap_denom,
                pair.denoms()
            )
        );
    }

    #[test]
    fn with_cl_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![7],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: denom {} not found in pool id {}",
                pair.quote_denom, pair.route[0]
            )
        );
    }

    #[test]
    fn with_cl_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![5, 7],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool {} contains no assets of the pool {}",
                pair.route[1], pair.route[0]
            )
        );
    }

    #[test]
    fn with_cl_pools_when_final_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![5, 7],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool {} contains no assets of the pool {}",
                pair.route[1], pair.route[0]
            )
        );
    }

    #[test]
    fn with_cl_pools_calculates_1_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![5],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 5,
                token_out_denom: DENOM_UOSMO.to_string(),
            }]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UOSMO.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 5,
                token_out_denom: DENOM_UATOM.to_string(),
            }]
        );
    }

    #[test]
    fn with_cl_pools_calculates_2_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![5, 6],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UION.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 5,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 6,
                    token_out_denom: DENOM_UION.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UION.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 6,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 5,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );
    }

    #[test]
    fn with_cl_pools_calculates_3_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![12, 11, 10],
            quote_denom: DENOM_USDC.to_string(),
            base_denom: DENOM_UATOM.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_USDC.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 12,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 12,
                    token_out_denom: DENOM_USDC.to_string(),
                },
            ]
        );
    }

    #[test]
    fn with_ss_pools_when_swap_denom_not_in_pair_denoms_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![10],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let swap_denom = "not_in_pair".to_string();

        let err = calculate_route(&deps.as_ref().querier, &pair, swap_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: swap denom {} not in pair denoms {:?}",
                swap_denom,
                pair.denoms()
            )
        );
    }

    #[test]
    fn with_ss_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![12],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: denom {} not found in pool id {}",
                pair.quote_denom, pair.route[0]
            )
        );
    }

    #[test]
    fn with_ss_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![10, 12],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool {} contains no assets of the pool {}",
                pair.route[1], pair.route[0]
            )
        );
    }

    #[test]
    fn with_ss_pools_when_final_pool_does_not_contain_target_denom_fails() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![10, 12],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        let err =
            calculate_route(&deps.as_ref().querier, &pair, pair.quote_denom.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Generic error: pool {} contains no assets of the pool {}",
                pair.route[1], pair.route[0]
            )
        );
    }

    #[test]
    fn with_ss_pools_calculates_1_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![10],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UOSMO.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 10,
                token_out_denom: DENOM_UOSMO.to_string(),
            }]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UOSMO.to_string()).unwrap(),
            vec![SwapAmountInRoute {
                pool_id: 10,
                token_out_denom: DENOM_UATOM.to_string(),
            }]
        );
    }

    #[test]
    fn with_ss_pools_calculates_2_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![10, 11],
            quote_denom: DENOM_UATOM.to_string(),
            base_denom: DENOM_UION.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UION.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UION.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );
    }

    #[test]
    fn with_ss_pools_calculates_3_pool_route() {
        let deps = calc_mock_dependencies();

        let pair = Pair {
            route: vec![12, 11, 10],
            quote_denom: DENOM_USDC.to_string(),
            base_denom: DENOM_UATOM.to_string(),
        };

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_USDC.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 12,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UATOM.to_string(),
                }
            ]
        );

        assert_eq!(
            calculate_route(&deps.as_ref().querier, &pair, DENOM_UATOM.to_string()).unwrap(),
            vec![
                SwapAmountInRoute {
                    pool_id: 10,
                    token_out_denom: DENOM_UOSMO.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 11,
                    token_out_denom: DENOM_UION.to_string(),
                },
                SwapAmountInRoute {
                    pool_id: 12,
                    token_out_denom: DENOM_USDC.to_string(),
                },
            ]
        );
    }
}
