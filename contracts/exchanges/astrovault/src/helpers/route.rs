#![allow(unused_variables, unused_imports)]

use crate::{state::{config::{get_config, get_router_config}, pairs::{find_pair, pair_exists}}, types::{pair::{Pair, PopulatedPair, StoredPairType}, route::{self, HopInfo, PopulatedRoute, Route, RouteHop, StoredRoute}, wrapper::ContractWrapper}, ContractError};
use astrovault::assets::{asset::{Asset, AssetInfo}, querier};
use cosmwasm_std::{ensure, from_json, to_json_binary, Binary, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, StdResult, Uint128};
use cw20::Cw20ReceiveMsg;
use astrovault::router::{
    state::{
        Hop as AstroHop, 
        Route as AstroRoute
    },
    handle_msg::ExecuteMsg as RouterExecute
};

use super::balance::to_asset_info;

pub fn reversed(route: &Route) -> Route {

    if route.len() == 1 {
        let hop = route.first().unwrap().clone();
        return vec![RouteHop {
            prev: hop.next.unwrap(),
            next: Some(hop.prev),
            denom: hop.denom,
        }]
    }


    let mut reversed: Vec<RouteHop> = Vec::with_capacity(route.len());

    let last = route.last().unwrap().clone();
    reversed.push(RouteHop {
        prev: last.next.unwrap(),
        next: None,
        denom: last.denom,
    });

    let hops_num = route.len();

    // take all but first in original
    for (index, hop) in route.iter().rev().enumerate().skip(1) {

        // git next hop in original route
        let orig_next = route.get(
            hops_num - index
        ).unwrap().clone();

        let next = if index == hops_num - 1 {
            Some(hop.prev.clone())
        } else {
            None
        };

        let rev_hop = RouteHop {
            prev: HopInfo {
                asset_info: to_asset_info(orig_next.denom),
                ..orig_next.prev
            },
            next,
            denom: hop.denom.clone(),
        };

        reversed.push(rev_hop);
    }

    reversed
}



pub fn route_denoms(
    route: &Route
) -> Vec<String> {
    route.iter().map(|h| h.denom.clone()).collect()
}


/// Return all denoms involved in the route
pub fn populated_route_denoms(
    route: &PopulatedRoute
) -> Vec<String> {

    let length = route.len();
    let mut route_denoms : Vec<String> = Vec::with_capacity(length - 1);

    // take all but last
    for (index, pool) in route.iter().enumerate().take(length - 1) {
        let next = route.get(index + 1).unwrap();

        if index == 0 {
            let combined = pool.combined_denoms(next);
            route_denoms.extend(combined);
        } else {
            let last_saved = route_denoms.last().unwrap();
            route_denoms.push(next.other_denom(last_saved));
        }

        /* if index == length - 2 {
            route_denoms.push(next.other_denom
        } */
    }

    route_denoms
}



pub fn route_pairs_to_astro_hops(
    querier:        &QuerierWrapper,
    route:          &PopulatedRoute,
    offer_info:     &AssetInfo,
) -> Result<Vec<AstroHop>, ContractError> {
    
    let mut astro_hops: Vec<AstroHop> = Vec::with_capacity(route.len());

    let first = route.first().unwrap();
    let last = route.last().unwrap();

    let mut offer_asset = offer_info.clone();

    for hop_pair in route {
        let astro_hop = hop_pair.astro_hop(querier, &offer_asset)?;
        astro_hops.push(astro_hop);
        offer_asset = hop_pair.other_asset(&offer_asset);
    }

    Ok(astro_hops)
}



pub fn route_swap_cosmos_msg(
    deps:           Deps,
    env:            Env,
    route_pair:     PopulatedPair,
    offer_asset:    Asset,
    target_asset:   Asset,
    funds:          Vec<Coin>,
) -> Result<CosmosMsg, ContractError> {

    let astro_hops = route_pairs_to_astro_hops(
        &deps.querier,
        &route_pair.route(),
        &offer_asset.info,
    )?;

    let route = AstroRoute {
        hops: astro_hops,
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
    route:                 PopulatedRoute,
    mut offer_asset:       Asset,
) -> StdResult<Uint128> {

    for pool in route {
        
        let amount = pool.swap_simulation(
            &deps.querier, 
            offer_asset.clone(),
        )?;

        let info = pool.other_asset(&offer_asset.info);

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
        helpers::{balance::{asset_to_coin, to_asset_info}, validated::{validated_route, validated_routed_pair}}, 
        state::{
            config::{get_config, update_config, update_router_config}, pairs::{find_pair, get_pairs, pair_exists, save_pair}, pools::pool_exists, routes::route_exists
        }, 
        tests::constants::{
            DCA_CONTRACT, DENOM_AARCH, DENOM_UATOM, DENOM_UNTRN, 
            DENOM_UOSMO, DENOM_USCRT, DENOM_UUSDC, ROUTER_CONTRACT
        }, 
        types::{
            config::Config, pair::{Pair, PopulatedPair, PopulatedPairType}, pool::{PoolType, PopulatedPool}, route::{HopInfo, PopulatedRoute, Route, RouteHop}
        }, 
        ContractError
    };
    use super::{populated_route_denoms, route_pairs_to_astro_hops, route_swap_cosmos_msg};
    use astrovault::router::{
        state::{
            Hop as AstroHop, 
            Route as AstroRoute
        },
        handle_msg::ExecuteMsg as RouterExecute
    };

    use crate::tests::common::{InitData, init, init_with_route, create_route, default_routed_pair};
    

    #[test]
    fn can_create_router_pair() {

        let data = init_with_route(vec![RouteHop {
            prev: HopInfo {
                address: "address".to_string(),
                pool_type: PoolType::Standard,
                asset_info: to_asset_info(DENOM_AARCH),
            },
            next: Some(HopInfo {
                address: "address".to_string(),
                pool_type: PoolType::Standard,
                asset_info: to_asset_info(DENOM_UUSDC),

            }),
            denom: DENOM_UOSMO.to_string(),
        }]);

        let deps = data.deps.as_ref();
        let pair = data.pair.unwrap();
    
        assert!(pair_exists(deps.storage, &pair.denoms()));
        assert_eq!(get_pairs(deps.storage, None, None).len(), 1);

        let route = validated_routed_pair(
            deps, 
            &pair, 
            Some(to_asset_info(DENOM_AARCH))
        ).unwrap().route();


        assert!(route.len() == 2);

        let first = route.first().unwrap();
        assert_eq!(first.base_asset, to_asset_info(DENOM_AARCH));
        assert_eq!(first.quote_asset, to_asset_info(DENOM_UOSMO));
        assert!(pool_exists(deps.storage, &first.denoms()));

        let second = route.last().unwrap();
        assert_eq!(second.base_asset, to_asset_info(DENOM_UOSMO));
        assert_eq!(second.quote_asset, to_asset_info(DENOM_UUSDC));
        assert!(pool_exists(deps.storage, &second.denoms()));

        assert!(route_exists(deps.storage, &[DENOM_AARCH.to_string(), DENOM_UUSDC.to_string()]));
    }



    #[test]
    fn astrohops_work() {

        let data = init_with_route(vec![
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_AARCH),
                },
                next: None,
                denom: DENOM_UOSMO.to_string(),
            },
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UOSMO),
                },
                next: None,
                denom: DENOM_USCRT.to_string(),
            },
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_USCRT),
                },
                next: Some(HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UUSDC),
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
        
        let mut pairs_hops = validated_routed_pair(
            deps, 
            &pair, 
            None
        ).unwrap().route();


        let astro_hops = route_pairs_to_astro_hops(
            &deps.querier,
            &pairs_hops,
            &offer.info,
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
            &deps.querier,
            &pairs_hops,
            &target.info,
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
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_AARCH),
                },
                next: None,
                denom: DENOM_UOSMO.to_string(),
            },
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UOSMO),
                },
                next: Some(HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UUSDC),
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

        
        let route_pair = validated_routed_pair(
            deps, 
            &pair, 
            Some(offer.info.clone())
        ).unwrap();


        let swap_funds = vec![asset_to_coin(offer.clone())];

        let swap_msg = route_swap_cosmos_msg(
            deps,
            env.clone(),
            route_pair,
            offer.clone(),
            target.clone(),
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
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_AARCH),
                },
                next: None,
                denom: DENOM_UOSMO.to_string(),
            },
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UOSMO),
                },
                next: None,
                denom: DENOM_UNTRN.to_string(),
            },
            RouteHop {
                prev: HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UNTRN),
                },
                next: Some(HopInfo {
                    address: "address".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UUSDC),
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
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_AARCH),
                    },
                    next: None,
                    denom: DENOM_UOSMO.to_string(),
                },
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UOSMO),
                    },
                    next: Some(HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UNTRN),
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(
            validated_routed_pair(
                deps, 
                &pair, 
                None
            ).unwrap_err(), 
            ContractError::RouteDublicates {}
        );


        // Okay
        let pair = Pair::new_routed(
            to_asset_info(DENOM_AARCH), 
            to_asset_info(DENOM_UNTRN),
            vec![
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_AARCH),
                    },
                    next: Some(HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UNTRN),
                    }),
                    denom: DENOM_UOSMO.to_string(),
                },
            ]
        );

        assert_eq!(validated_routed_pair(
            deps, 
            &pair, 
            None
        ).unwrap().route().len(), 2);



        // Reverse

        // Wrong previous
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_USCRT),
                    },
                    next: Some(HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UOSMO),
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(
            validated_routed_pair(
                deps, 
                &pair, 
                Some(to_asset_info(DENOM_UOSMO))
            ).unwrap_err(), 
            ContractError::InvalidHops{}
        );

        // Wrong next
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UUSDC),
                    },
                    next: Some(HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_USCRT),
                    }),
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(
            validated_routed_pair(
                deps, 
                &pair, 
                Some(to_asset_info(DENOM_UOSMO))
            ).unwrap_err(), 
            ContractError::InvalidHops{}
        );


        // Missing next
        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UUSDC),
                    },
                    next: None,
                    denom: DENOM_UNTRN.to_string(),
                },
            ]
        );

        assert_eq!(
            validated_routed_pair(
                deps, 
                &pair, 
                None
            ).unwrap_err(), 
            ContractError::MissingNextPoolHop{}
        );

    

        let pair = Pair::new_routed(
            to_asset_info(DENOM_UUSDC),
            to_asset_info(DENOM_UOSMO), 
            vec![
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UUSDC),
                    },
                    next: None,
                    denom: DENOM_USCRT.to_string(),
                },
                RouteHop {
                    prev: HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_USCRT),
                    },
                    next: Some(HopInfo {
                        address: "address".to_string(),
                        pool_type: PoolType::Standard,
                        asset_info: to_asset_info(DENOM_UOSMO),
                    }),
                    denom: DENOM_UNTRN.to_string(),
                }
            ]
        );
        
        

        let validated = validated_routed_pair(
            deps, 
            &pair, 
            None
        ).unwrap();

        let route = validated.route();
        assert_eq!(route.len(), 3);

        // pairs have been created earlier so the base <-> quote is swapped with original
        let first = route.first().unwrap();
        assert_eq!(first.quote_asset, pair.base_asset);
        assert_eq!(first.quote_asset, to_asset_info(DENOM_UUSDC));
        assert_eq!(first.base_asset, to_asset_info(DENOM_USCRT));

        let last = route.last().unwrap();
        assert_eq!(last.quote_asset, to_asset_info(DENOM_UNTRN));
        assert_eq!(last.base_asset, to_asset_info(DENOM_UOSMO));
        assert_eq!(last.base_asset, pair.quote_asset);


        // not base and not quote
        assert_eq!(
            validated_routed_pair(
                deps, 
                &pair, 
                Some(to_asset_info(DENOM_USCRT))
            ).unwrap_err(), 
            ContractError::RouteNotFound{}
        );


        let reversed = validated_routed_pair(
            deps, 
            &pair, 
            Some(to_asset_info(DENOM_UOSMO))
        ).unwrap();

        let route = reversed.route();
        assert_eq!(route.first().unwrap(), last);
        assert_eq!(route.last().unwrap(),first);
    }

  

    #[test]
    fn routed_simulation_works() {
        let data = init_with_route(vec![
            RouteHop {
                prev: HopInfo {
                    address: "arch_scrt_pair".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_AARCH),
                },
                next: Some(HopInfo {
                    address: "scrt_usdc_pair".to_string(),
                    pool_type: PoolType::Standard,
                    asset_info: to_asset_info(DENOM_UUSDC),
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
    fn populated_denoms_work () {
      let denoms = populated_route_denoms(&default_routed_pair().route());
      assert_eq!(vec![
          "A".to_string(),
          "B".to_string(),
          "C".to_string(),
          "D".to_string(),
          "E".to_string(),
          "F".to_string(),
      ], denoms);
    }






















}