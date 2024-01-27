#![allow(unused_variables, unused_imports)]

use crate::{state::{config::{get_config, get_router_config}, pairs::{find_route_pair, pair_exists, route_pair_exists}}, types::{pair::{Pair, PairRoute, PoolInfo}, position_type::PositionType, wrapper::ContractWrapper}, ContractError};
use astrovault::assets::asset::{Asset, AssetInfo};
use cosmwasm_std::{ensure, from_json, to_json_binary, Binary, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, StdResult, Uint128};
use cw20::Cw20ReceiveMsg;
use super::{pair, pool::{self, validated_direct_pair}};
use astrovault::router::{
    state::{
        Hop as AstroHop, 
        Route as AstroRoute
    },
    handle_msg::ExecuteMsg as RouterExecute
};



fn validated_route_pair(
    deps:               Deps,
    pair:               &Pair,
    allow_missing:      bool,
) -> Result<(Pair, bool), ContractError> {
    let found =  find_route_pair(deps.storage, pair.denoms());
    
    match found {
        Ok(pair) => Ok((pair, true)), 
        Err(_) => {
            if allow_missing {
                return Ok((validated_direct_pair(deps, pair)?, false));
            } else {
                return Err(ContractError::NoRoutedPair {});
            }
        }
    }
}


/// check that the route is valid, populate indexes and return a list of (pool) pairs to be saved
pub fn validated_route_pairs(
    deps:                   Deps,
    pair:                   &Pair,
    allow_missing:          bool,
) -> Result<Vec<Pair>, ContractError> {
    pair.valid_route_hops()?;

    pair
    .route_pairs()
    .iter()
    .map(|pair| {
        let val = validated_route_pair(deps, pair, allow_missing)?;
        Ok(val.0)
    })
    .collect::<Result<Vec<Pair>, ContractError>>()
}



pub fn validated_route_pairs_to_save(
    deps:                   Deps,
    pair:                   &Pair,
) -> Result<Vec<Pair>, ContractError> {
    pair.valid_route_hops()?;

    pair
    .route_pairs()
    .iter()
    .map(|pair| validated_route_pair(deps, pair, true))
    .filter_map(|res| {
        match res {
            Ok((pair, existed)) => {
                if existed {
                    None
                } else {
                    Some(Ok(pair))
                }
            },
            Err(err) => Some(Err(err))
        }
    })
    .collect::<Result<Vec<Pair>, ContractError>>()
}


pub fn route_pairs_to_astro_hops(
    deps:           Deps,
    pair_hops:      &Vec<Pair>,
    offer_asset:    &Asset,
    target_info:    &AssetInfo,
) -> Result<Vec<AstroHop>, ContractError> {
    let mut astro_hops: Vec<AstroHop> = Vec::with_capacity(pair_hops.len());

    let first = pair_hops.first().unwrap();
    let last = pair_hops.last().unwrap();

    let mut offer_asset = offer_asset.info.clone();

    ensure!(first.base_asset == offer_asset || 
            first.quote_asset == offer_asset, ContractError::RouteRuntimeError {});


    for hop_pair in pair_hops {
        let astro_hop = hop_pair.to_astro_hop(&deps.querier, &offer_asset)?;
        astro_hops.push(astro_hop);

        offer_asset = hop_pair.other_asset(&offer_asset);
        
        if hop_pair.eq(last) {
            ensure!(offer_asset == *target_info, ContractError::RouteRuntimeError {});
        }
    }

    Ok(astro_hops)
}




pub fn route_swap_cosmos_msg(
    deps:                Deps,
    env:                 Env,
    pair:                &Pair,
    offer_asset:         Asset,
    target_asset:        Asset,
    route:               Option<Binary>,
    funds:               Vec<Coin>,
) -> Result<CosmosMsg, ContractError> {

    let pair = match route {
        Some(route_binary) => {
            let parsed_route: PairRoute = from_json(&route_binary)?;
            Pair::new_routed(
                pair.base_asset.clone(), 
                pair.quote_asset.clone(), 
                parsed_route
            )
        },
        None => pair.clone()
    };


    let mut hops_pairs = validated_route_pairs(
        deps,
        &pair,
        false
    )?;
    

    if !offer_asset.info.equal(&pair.base_asset) {
        hops_pairs.reverse();
    }


    let hops = route_pairs_to_astro_hops(
        deps,
        &hops_pairs,
        &offer_asset,
        &target_asset.info,
    )?;


    let route = AstroRoute {
        hops,
        minimum_receive: Some(target_asset.amount),
        to: None,
    };

    let route_binary = to_json_binary(&route)?;

    let cfg = get_config(deps.storage)?;
    
    let router = ContractWrapper(cfg.router_address.into());

    let msg = if offer_asset.info.is_native_token() {
        router.execute(
            to_json_binary(&RouterExecute::Receive(Cw20ReceiveMsg {
                sender: env.contract.address.to_string(),
                amount: offer_asset.amount,
                msg: route_binary,
            }))?, 
            funds
        )?
    } else {
        router.execute_cw20(
            offer_asset.to_string(), 
            offer_asset.amount, 
            route_binary
        )?
    };

    Ok(msg)

}



pub fn get_route_swap_simulate(
    deps:                  Deps,
    pair:                  &Pair,
    offer_asset:           Asset,
) -> StdResult<Uint128> {

    let mut hops_pairs = validated_route_pairs(
        deps,
        pair,
        false
    ).map_err(|e| StdError::generic_err(e.to_string()))?;
    

    if !offer_asset.info.equal(&pair.base_asset) {
        hops_pairs.reverse();
    }

    let mut offer_asset = offer_asset.clone();

    for hop_pair in hops_pairs {
        
        let amount = hop_pair.pool_info().get_swap_simulate(
            &deps.querier, 
            offer_asset.clone(),
        )?;

        let info = hop_pair.other_asset(&offer_asset.info);

        offer_asset = Asset {
            info,
            amount,
        };
    }

    Ok(offer_asset.amount)
}



#[cfg(test)]
mod creating_routed_pairs_tests {
    use std::{env, vec};
    use astrovault::{
        assets::asset::{Asset, AssetInfo}, 
        standard_pool::query_msg::{QueryMsg as StandardQuery, SimulationResponse}
    };
    use cosmwasm_std::{
        from_json, to_json_binary,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier}, 
        Addr, Coin, ContractResult, CosmosMsg, DepsMut, Empty, Env, MemoryStorage, 
        MessageInfo, OwnedDeps, StdError, SystemError, SystemResult, Uint128, WasmMsg, WasmQuery
    };
    use cw20::Cw20ReceiveMsg;
    use crate::{
        handlers::{
            create_pairs::create_pairs_handler, 
            get_expected_receive_amount::get_expected_receive_amount_handler
        }, 
        helpers::{
            balance::{asset_to_coin, to_asset_info}, 
            route::validated_route_pairs
        }, 
        state::{
            config::{get_config, update_config, update_router_config}, 
            pairs::{find_pair, get_pairs, pair_exists, route_pair_exists}
        }, 
        tests::constants::{
            DCA_CONTRACT, DENOM_AARCH, DENOM_UATOM, DENOM_UNTRN, 
            DENOM_UOSMO, DENOM_USCRT, DENOM_UUSDC, ROUTER_CONTRACT
        }, 
        types::{
            config::Config, 
            pair::{HopInfo, HopSide, Pair, PairRoute, PairType, PoolType}
        }, 
        ContractError
    };
    use super::{route_pairs_to_astro_hops, route_swap_cosmos_msg};
    use astrovault::router::{
        state::{
            Hop as AstroHop, 
            Route as AstroRoute
        },
        handle_msg::ExecuteMsg as RouterExecute
    };
    


    struct InitData {
        pub deps:           OwnedDeps<MemoryStorage, MockApi, MockQuerier, Empty>,
        pub env:            Env,
        pub admin_info:     MessageInfo,
        pub pair:           Option<Pair>
    }

    fn init() -> InitData  {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let admin_info = mock_info("admin", &[]);
        let deps_mut = deps.as_mut();

        update_config(deps_mut.storage, Config {
            admin: Addr::unchecked("admin"),
            dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            router_address: Addr::unchecked(ROUTER_CONTRACT),
        }).unwrap();


        InitData {
            deps,
            env,
            admin_info,
            pair: None
        }
    }

    fn create_route(
        deps: DepsMut, 
        info: MessageInfo,
        route: PairRoute
    ) -> Pair {
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UUSDC),
            route
        );

        create_pairs_handler(
            deps,
            info,
            vec![pair.clone()],
        ).unwrap();

        pair
    }


    fn init_with_route(route: PairRoute) -> InitData  {
        let mut data = init();

        let pair = create_route(
            data.deps.as_mut(),
            data.admin_info.clone(),
            route
        );
        
        InitData {
            pair: Some(pair),
            ..data
        }
    }


    #[test]
    fn can_create_router_pair() {

        let data = init_with_route(vec![HopInfo {
            prev: HopSide {
                address: "address".to_string(),
                pool_type: PoolType::Standard,
            },
            next: Some(HopSide {
                address: "address".to_string(),
                pool_type: PoolType::Standard,
            }),
            denom: DENOM_UOSMO.to_string(),
        }]);

        let deps = data.deps.as_ref();
        let pair = data.pair.unwrap();
    
        assert!(pair_exists(deps.storage, &pair));
        assert_eq!(get_pairs(deps.storage, None, None).len(), 1);

        let pairs = validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap();

        assert!(pairs.len() == 2);

        let first = pairs.first().unwrap();
        assert_eq!(first.base_asset, to_asset_info(DENOM_AARCH));
        assert_eq!(first.quote_asset, to_asset_info(DENOM_UOSMO));
        assert!(route_pair_exists(deps.storage, first.denoms()));

        let second = pairs.last().unwrap();
        assert_eq!(second.base_asset, to_asset_info(DENOM_UOSMO));
        assert_eq!(second.quote_asset, to_asset_info(DENOM_UUSDC));
        assert!(route_pair_exists(deps.storage, second.denoms()));
    }



    #[test]
    fn astrohops_work() {

        let data = init_with_route(vec![
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UOSMO.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_USCRT.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UNTRN.to_string(),
            }
        ]);

        let deps = data.deps.as_ref();
        let pair = data.pair.unwrap();


        let offer = Asset {
            info: to_asset_info(DENOM_AARCH),
            amount: Uint128::new(10_000_000),
        };

        let target = Asset {
            info: to_asset_info(DENOM_UUSDC),
            amount: Uint128::new(1_000_000),
        };

        let mut pairs_hops = pair.route_pairs();

        let astro_hops = route_pairs_to_astro_hops(
            deps,
            &pairs_hops,
            &offer,
            &target.info,
        ).unwrap();

        assert!(astro_hops.len() == 4);

        let first = astro_hops.first().unwrap().clone();
        let first_info = first.standard_hop_info.unwrap();
        assert_eq!(first_info.offer_asset_info, offer.info);
        assert_eq!(first_info.ask_asset_info, to_asset_info(DENOM_UOSMO));


        let last = astro_hops.last().unwrap().clone();
        let last_info = last.standard_hop_info.unwrap();
        assert_eq!(last_info.offer_asset_info, to_asset_info(DENOM_UNTRN));
        assert_eq!(last_info.ask_asset_info, target.info);


        // reverse
        pairs_hops.reverse();

        let astro_hops = route_pairs_to_astro_hops(
            deps,
            &pairs_hops,
            &target,
            &offer.info,
        ).unwrap();

        assert!(astro_hops.len() == 4);

        let first = astro_hops.first().unwrap().clone();
        let first_info = first.standard_hop_info.unwrap();
        assert_eq!(first_info.offer_asset_info, target.info);
        assert_eq!(first_info.ask_asset_info, to_asset_info(DENOM_UNTRN));

        let third = astro_hops.get(2).unwrap().clone();
        let third_info = third.standard_hop_info.unwrap();
        assert_eq!(third_info.offer_asset_info, to_asset_info(DENOM_USCRT));
        assert_eq!(third_info.ask_asset_info, to_asset_info(DENOM_UOSMO));


        let last = astro_hops.last().unwrap().clone();
        let last_info = last.standard_hop_info.unwrap();
        assert_eq!(last_info.offer_asset_info, to_asset_info(DENOM_UOSMO));
        assert_eq!(last_info.ask_asset_info, offer.info);
    }


    #[test]
    fn astroroute_msg_work() {

        let data = init_with_route(vec![
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UOSMO.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UNTRN.to_string(),
            }
        ]);

        let deps = data.deps.as_ref();
        let env = data.env;
        let pair = data.pair.unwrap();
        

        let offer = Asset {
            info: to_asset_info(DENOM_AARCH),
            amount: Uint128::new(10_000_000),
        };

        let target = Asset {
            info: to_asset_info(DENOM_UUSDC),
            amount: Uint128::new(1_000_000),
        };

        let pairs_hops = pair.route_pairs();
        let swap_funds = vec![asset_to_coin(offer.clone())];


        let swap_msg = route_swap_cosmos_msg(
            deps,
            env.clone(),
            &pair,
            offer.clone(),
            target.clone(),
            None,
            swap_funds.clone()
        ).unwrap();

        let cfg = get_config(deps.storage).unwrap();


        let msg = if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) = swap_msg {
            assert_eq!(contract_addr, cfg.router_address);
            assert_eq!(swap_funds, funds);
            msg            
        } else {
            panic!("wrong execute message")
        };


        let msg = if let RouterExecute::Receive(cw20) =  from_json(&msg).unwrap() {
            assert_eq!(cw20.sender, env.contract.address.to_string());
            assert_eq!(cw20.amount, offer.amount);
            cw20.msg
        } else {
            panic!("wrong swap message")
        };


        let astro_route: AstroRoute = from_json(&msg).unwrap();
        assert_eq!(astro_route.hops.len(), 3);
        assert_eq!(astro_route.minimum_receive.unwrap(), target.amount);
    }


    #[test]
    fn pairs_from_initial_route_works() {

        let data = init_with_route(vec![
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UOSMO.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UNTRN.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_USCRT.to_string(),
            },
        ]);

        let deps = data.deps.as_ref();

        // not saving any following pairs

        // UNTRN repeating
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UNTRN),
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UOSMO.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), StdError::generic_err("Route denoms are not unique").into());


        // Okay
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UNTRN),
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UOSMO.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap().len(), 2);



        // Reverse

        // Skipping SCRT
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), ContractError::NoRoutedPair {});


        // Wring order
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_USCRT.to_string(),
                },
            ]
        );


        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), ContractError::NoRoutedPair {});


        // Ok
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_USCRT.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                }
            ]
        );

    }

  

    #[test]
    fn routed_simulation_works() {
        let data = init_with_route(vec![
            HopInfo {
                prev: HopSide {
                    address: "arch_scrt_pair".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "scrt_usdc_pair".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_USCRT.to_string(),
            },
        ]);

        let mut deps = data.deps;


        let offer = Asset {
            info: to_asset_info(DENOM_AARCH),
            amount: Uint128::new(10_000_000),
        };

        let target_info : AssetInfo = to_asset_info(DENOM_UUSDC);


        deps.querier.update_wasm(|query| {

            let msg = if let WasmQuery::Smart { contract_addr, msg } = query {
                assert!(contract_addr == "arch_scrt_pair" || contract_addr == "scrt_usdc_pair");
                msg
            } else {
                panic!("wrong query type")
            };

            let offer = if let StandardQuery::Simulation { offer_asset } = from_json(&msg).unwrap() {
                offer_asset
            } else {
                panic!("wrong query variant")
            };


            if offer.info.equal(&to_asset_info(DENOM_AARCH))  {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&SimulationResponse {
                        return_amount:      Uint128::from(5_000_000u128),
                        spread_amount:      Uint128::default(),
                        commission_amount:  Uint128::default(),
                        buybackburn_amount: Uint128::default(),
                    })
                    .unwrap(),
                ))
            } else if offer.info.equal(&to_asset_info(DENOM_USCRT)) {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&SimulationResponse {
                        return_amount:      Uint128::from(2_000_000u128),
                        spread_amount:      Uint128::default(),
                        commission_amount:  Uint128::default(),
                        buybackburn_amount: Uint128::default(),
                    })
                    .unwrap(),
                ))
            } else {
                SystemResult::Err(SystemError::UnsupportedRequest { kind: "bad kind".into() })
            }
        });


        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                asset_to_coin(offer.clone()),
                DENOM_UUSDC.to_string()
            )
            .unwrap(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: Uint128::from(2_000_000u128)
            }
        )
    }















    

    #[test]
    fn pairs_overriding_and_counts() {

        let data = init_with_route(vec![
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UOSMO.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_UNTRN.to_string(),
            },
            HopInfo {
                prev: HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                },
                next: Some(HopSide {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                }),
                denom: DENOM_USCRT.to_string(),
            },
        ]);

        let deps = data.deps.as_ref();

        // not saving any following pairs

        // UNTRN repeating
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UNTRN),
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UOSMO.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), StdError::generic_err("Route denoms are not unique").into());


        // Okay
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UNTRN),
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UOSMO.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap().len(), 2);



        // Reverse

        // Skipping SCRT
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), ContractError::NoRoutedPair {});


        // Wring order
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_USCRT.to_string(),
                },
            ]
        );


        assert_eq!(validated_route_pairs(
            deps, 
            &pair, 
            false
        ).unwrap_err(), ContractError::NoRoutedPair {});


        // Ok
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_USCRT.to_string(),
                },
                HopInfo {
                    prev: HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    },
                    next: Some(HopSide {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                    }),
                    denom: DENOM_UNTRN.to_string(),
                }
            ]
        );

    }










}