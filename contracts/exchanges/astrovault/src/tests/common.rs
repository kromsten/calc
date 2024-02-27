use crate::{
    handlers::create_pairs::create_pairs_handler,
    helpers::balance::to_asset_info,
    state::config::update_config,
    tests::constants::{DCA_CONTRACT, DENOM_AARCH, DENOM_UUSDC, ROUTER_CONTRACT},
    types::{
        config::Config,
        pair::{Pair, PopulatedPair},
        pool::{PoolType, PopulatedPool},
        route::{HopInfo, Route, RouteHop},
    },
};
use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
    Addr, DepsMut, Empty, Env, MemoryStorage, MessageInfo, OwnedDeps,
};
use std::vec;

pub struct InitData {
    pub deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier, Empty>,
    pub env: Env,
    pub admin_info: MessageInfo,
    pub pair: Option<Pair>,
}

pub fn init() -> InitData {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let admin_info = mock_info("admin", &[]);
    let deps_mut = deps.as_mut();

    update_config(
        deps_mut.storage,
        Config {
            admin: Addr::unchecked("admin"),
            dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            router_address: Addr::unchecked(ROUTER_CONTRACT),
        },
    )
    .unwrap();

    InitData {
        deps,
        env,
        admin_info,
        pair: None,
    }
}

pub fn init_real_implicit() -> InitData {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let admin_info = mock_info("admin", &[]);
    let deps_mut = deps.as_mut();

    update_config(
        deps_mut.storage,
        Config {
            admin: Addr::unchecked("admin"),
            dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            router_address: Addr::unchecked(ROUTER_CONTRACT),
        },
    )
    .unwrap();

    for pair in routed_pairs_real_implicit() {
        let deps_mut = deps.as_mut();

        create_pairs_handler(deps_mut, admin_info.clone(), vec![pair.clone()]).unwrap();
    }

    InitData {
        deps,
        env,
        admin_info,
        pair: None,
    }
}

pub fn create_route(deps: DepsMut, info: MessageInfo, route: Route) -> Pair {
    let pair = Pair::new_routed(
        to_asset_info(DENOM_AARCH),
        to_asset_info(DENOM_UUSDC),
        route,
    );

    create_pairs_handler(deps, info, vec![pair.clone()]).unwrap();

    pair
}

pub fn init_with_route(route: Route) -> InitData {
    let mut data = init();

    let pair = create_route(data.deps.as_mut(), data.admin_info.clone(), route);

    InitData {
        pair: Some(pair),
        ..data
    }
}

pub fn default_routed_pair() -> PopulatedPair {
    PopulatedPair::from_assets_routed(
        AssetInfo::NativeToken {
            denom: format!("A"),
        },
        AssetInfo::NativeToken {
            denom: format!("F"),
        },
        vec![
            PopulatedPool::from_assets(
                AssetInfo::NativeToken {
                    denom: format!("A"),
                },
                AssetInfo::NativeToken {
                    denom: format!("B"),
                },
            ),
            PopulatedPool::from_assets(
                AssetInfo::NativeToken {
                    denom: format!("B"),
                },
                AssetInfo::NativeToken {
                    denom: format!("C"),
                },
            ),
            PopulatedPool::from_assets(
                AssetInfo::NativeToken {
                    denom: format!("C"),
                },
                AssetInfo::NativeToken {
                    denom: format!("D"),
                },
            ),
            PopulatedPool::from_assets(
                AssetInfo::NativeToken {
                    denom: format!("D"),
                },
                AssetInfo::NativeToken {
                    denom: format!("E"),
                },
            ),
            PopulatedPool::from_assets(
                AssetInfo::NativeToken {
                    denom: format!("E"),
                },
                AssetInfo::NativeToken {
                    denom: format!("F"),
                },
            ),
        ],
    )
}

pub fn routed_pairs_real_implicit() -> Vec<Pair> {
    vec![
        Pair::new_routed(
            AssetInfo::NativeToken {
                denom: "aconst".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"
                    .to_string(),
            },
            vec![RouteHop {
                denom: "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"
                    .to_string(),
                prev: HopInfo {
                    address: "archway1flsdgve559shl8fvqfrk3p2wdxn82ykqyqk7w2wjaw7p3gnhj3esfmq8t0"
                        .to_string(),
                    pool_type: PoolType::Stable, // Assuming PoolType is defined and filled correctly
                    asset_info: AssetInfo::NativeToken {
                        denom: "aconst".to_string(),
                    },
                },
                next: Some(HopInfo {
                    address: "archway1903dqer5mdy4wen9duxhm7l76gw20vzk2vwm6t7zk305c0m38ldqjncc9f"
                        .to_string(),
                    pool_type: PoolType::Ratio, // Assuming PoolType is defined and filled correctly
                    asset_info: AssetInfo::Token {
                        contract_addr:
                            "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"
                                .to_string(),
                    },
                }),
            }],
        ),
        Pair::new_routed(
            AssetInfo::Token {
                contract_addr: "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"
                    .to_string(),
            },
            AssetInfo::Token {
                contract_addr: "archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf"
                    .to_string(),
            },
            vec![
                RouteHop {
                    denom: "archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm"
                        .to_string(),
                    prev: HopInfo {
                        address:
                            "archway1jdrvvzd2tcfvhvyaedy7e8s92lh2m3a3jklvn74768fh6n5quh4sl6rgkx"
                                .to_string(),
                        pool_type: PoolType::Standard, // Assuming PoolType is defined and filled correctly
                        asset_info: AssetInfo::Token {
                            contract_addr:
                                "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"
                                    .to_string(),
                        },
                    },
                    next: None,
                },
                RouteHop {
                    denom: "archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s"
                        .to_string(),
                    prev: HopInfo {
                        address:
                            "archway1gzumcx38q446k9mup02cvmc4xjrlwa6xhnram3lghw2te8zeu7rqndhcrq"
                                .to_string(),
                        pool_type: PoolType::Stable, // Assuming PoolType is defined and filled correctly
                        asset_info: AssetInfo::Token {
                            contract_addr:
                                "archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm"
                                    .to_string(),
                        },
                    },
                    next: Some(HopInfo {
                        address:
                            "archway1gzumcx38q446k9mup02cvmc4xjrlwa6xhnram3lghw2te8zeu7rqndhcrq"
                                .to_string(),
                        pool_type: PoolType::Stable, // Assuming PoolType is defined and filled correctly
                        asset_info: AssetInfo::Token {
                            contract_addr:
                                "archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf"
                                    .to_string(),
                        },
                    }),
                },
            ],
        ),
    ]
}
