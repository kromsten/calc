use super::old_mocks::{MockApp, ADMIN, DENOM_UKUJI, DENOM_UTEST, FEE_COLLECTOR, USER};
use crate::{
    constants::{ONE, TEN},
    contract::instantiate,
    handlers::get_vault::get_old_vault_handler,
    msg::{EventsResponse, InstantiateMsg, QueryMsg, VaultResponse},
    state::{
        config::FeeCollector,
        old_cache::{Cache, OLD_CACHE},
        old_pairs::PAIRS,
        old_triggers::save_old_trigger,
        old_vaults::{save_old_vault, update_old_vault},
    },
    types::{dca_plus_config::DcaPlusConfig, old_vault::OldVault, vault_builder::VaultBuilder},
};
use base::{
    events::event::Event,
    pair::Pair,
    triggers::trigger::{OldTimeInterval, OldTrigger, OldTriggerConfiguration},
    vaults::vault::{OldDestination, OldVaultStatus, PostExecutionAction},
};
use cosmwasm_std::{
    from_binary,
    testing::{MockApi, MockQuerier},
    to_binary, Addr, Coin, ContractResult, Decimal, DepsMut, Env, MemoryStorage, MessageInfo,
    OwnedDeps, SystemResult, Timestamp, Uint128, WasmQuery,
};
use fin_helpers::msg::{FinBookResponse, FinPoolResponseWithoutDenom};
use kujira::fin::QueryMsg as FinQueryMsg;
use std::str::FromStr;

pub fn instantiate_contract(deps: DepsMut, env: Env, info: MessageInfo) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors: vec![FeeCollector {
            address: ADMIN.to_string(),
            allocation: Decimal::from_str("1").unwrap(),
        }],
        swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        staking_router_address: Addr::unchecked(ADMIN),
        page_limit: 1000,
        paused: false,
        dca_plus_escrow_level: Decimal::from_str("0.0075").unwrap(),
    };

    instantiate(deps, env.clone(), info.clone(), instantiate_message).unwrap();
}

pub fn instantiate_contract_with_community_pool_fee_collector(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors: vec![
            FeeCollector {
                address: FEE_COLLECTOR.to_string(),
                allocation: Decimal::from_str("0.5").unwrap(),
            },
            FeeCollector {
                address: "community_pool".to_string(),
                allocation: Decimal::from_str("0.5").unwrap(),
            },
        ],
        swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        staking_router_address: Addr::unchecked(ADMIN),
        page_limit: 1000,
        paused: false,
        dca_plus_escrow_level: Decimal::from_str("0.0075").unwrap(),
    };

    instantiate(deps, env.clone(), info.clone(), instantiate_message).unwrap();
}

pub fn instantiate_contract_with_multiple_fee_collectors(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_collectors: Vec<FeeCollector>,
) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors,
        swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        staking_router_address: Addr::unchecked(ADMIN),
        page_limit: 1000,
        paused: false,
        dca_plus_escrow_level: Decimal::from_str("0.0075").unwrap(),
    };

    instantiate(deps, env.clone(), info.clone(), instantiate_message).unwrap();
}

impl Default for OldVault {
    fn default() -> Self {
        Self {
            id: Uint128::zero(),
            created_at: Timestamp::default(),
            owner: Addr::unchecked(USER),
            label: Some("vault".to_string()),
            destinations: vec![OldDestination {
                address: Addr::unchecked(USER),
                allocation: Decimal::percent(100),
                action: PostExecutionAction::Send,
            }],
            status: OldVaultStatus::Active,
            balance: Coin::new(TEN.into(), DENOM_UKUJI),
            swap_amount: ONE,
            pair: Pair {
                address: Addr::unchecked("pair"),
                base_denom: DENOM_UKUJI.to_string(),
                quote_denom: DENOM_UTEST.to_string(),
            },
            slippage_tolerance: None,
            minimum_receive_amount: None,
            time_interval: OldTimeInterval::Daily,
            started_at: None,
            swapped_amount: Coin::new(0, DENOM_UKUJI),
            received_amount: Coin::new(0, DENOM_UTEST),
            trigger: Some(OldTriggerConfiguration::Time {
                target_time: Timestamp::from_seconds(0),
            }),
            dca_plus_config: None,
        }
    }
}

impl Default for DcaPlusConfig {
    fn default() -> Self {
        Self {
            escrow_level: Decimal::percent(10),
            model_id: 30,
            total_deposit: Coin::new(TEN.into(), DENOM_UKUJI),
            standard_dca_swapped_amount: Coin::new(0, DENOM_UKUJI),
            standard_dca_received_amount: Coin::new(0, DENOM_UTEST),
            escrowed_balance: Coin::new(0, DENOM_UTEST),
        }
    }
}

pub fn setup_old_vault(deps: DepsMut, env: Env, vault: OldVault) -> OldVault {
    PAIRS
        .save(deps.storage, vault.pair.address.clone(), &vault.pair)
        .unwrap();

    update_old_vault(deps.storage, &vault).unwrap();

    if vault.trigger.is_some() {
        save_old_trigger(
            deps.storage,
            OldTrigger {
                vault_id: vault.id,
                configuration: OldTriggerConfiguration::Time {
                    target_time: env.block.time,
                },
            },
        )
        .unwrap();
    }

    get_old_vault_handler(deps.as_ref(), vault.id)
        .unwrap()
        .vault
}

pub fn setup_vault(
    deps: DepsMut,
    env: Env,
    balance: Uint128,
    swap_amount: Uint128,
    status: OldVaultStatus,
    is_dca_plus: bool,
) -> OldVault {
    let pair = Pair {
        address: Addr::unchecked("pair"),
        base_denom: DENOM_UKUJI.to_string(),
        quote_denom: DENOM_UTEST.to_string(),
    };

    PAIRS
        .save(deps.storage, pair.address.clone(), &pair)
        .unwrap();

    let owner = Addr::unchecked("owner");

    let vault = save_old_vault(
        deps.storage,
        VaultBuilder {
            owner: owner.clone(),
            label: None,
            destinations: vec![OldDestination {
                address: owner,
                allocation: Decimal::percent(100),
                action: PostExecutionAction::ZDelegate,
            }],
            created_at: env.block.time.clone(),
            status,
            pair,
            swap_amount,
            position_type: None,
            slippage_tolerance: None,
            minimum_receive_amount: None,
            balance: Coin::new(balance.into(), DENOM_UKUJI),
            time_interval: OldTimeInterval::Daily,
            started_at: None,
            swapped_amount: Coin {
                denom: DENOM_UKUJI.to_string(),
                amount: Uint128::new(0),
            },
            received_amount: Coin {
                denom: DENOM_UTEST.to_string(),
                amount: Uint128::new(0),
            },
            dca_plus_config: if is_dca_plus {
                Some(DcaPlusConfig::new(
                    Decimal::percent(5),
                    30,
                    Coin::new(balance.into(), DENOM_UKUJI),
                    DENOM_UTEST.to_string(),
                ))
            } else {
                None
            },
        },
    )
    .unwrap();

    save_old_trigger(
        deps.storage,
        OldTrigger {
            vault_id: vault.id,
            configuration: OldTriggerConfiguration::Time {
                target_time: env.block.time,
            },
        },
    )
    .unwrap();

    OLD_CACHE
        .save(
            deps.storage,
            &Cache {
                vault_id: vault.id,
                owner: Addr::unchecked("owner"),
            },
        )
        .unwrap();

    vault
}

pub fn setup_active_vault_with_funds(deps: DepsMut, env: Env) -> OldVault {
    setup_vault(deps, env, TEN, ONE, OldVaultStatus::Active, false)
}

pub fn setup_active_dca_plus_vault_with_funds(deps: DepsMut, env: Env) -> OldVault {
    setup_vault(deps, env, TEN, ONE, OldVaultStatus::Active, true)
}

pub fn setup_active_vault_with_slippage_funds(deps: DepsMut, env: Env) -> OldVault {
    setup_vault(
        deps,
        env,
        Uint128::new(500000),
        Uint128::new(500000),
        OldVaultStatus::Active,
        false,
    )
}

pub fn setup_active_vault_with_low_funds(deps: DepsMut, env: Env) -> OldVault {
    setup_vault(
        deps,
        env,
        Uint128::new(10),
        Uint128::new(100),
        OldVaultStatus::Active,
        false,
    )
}

pub fn setup_active_dca_plus_vault_with_low_funds(
    deps: DepsMut,
    env: Env,
    balance: Uint128,
    swap_amount: Uint128,
) -> OldVault {
    setup_vault(
        deps,
        env,
        balance,
        swap_amount,
        OldVaultStatus::Active,
        true,
    )
}

pub fn assert_address_balances(mock: &MockApp, address_balances: &[(&Addr, &str, Uint128)]) {
    address_balances
        .iter()
        .for_each(|(address, denom, expected_balance)| {
            assert_eq!(
                mock.get_balance(address, denom),
                expected_balance,
                "Balance mismatch for {} at {}",
                address,
                denom
            );
        })
}

pub fn assert_events_published(mock: &MockApp, resource_id: Uint128, expected_events: &[Event]) {
    let events_response: EventsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &QueryMsg::GetEventsByResourceId {
                resource_id,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    expected_events.iter().for_each(|expected_event| {
        assert!(
            events_response.events.contains(expected_event),
            "Expected actual_events: \n\n{:?}\n\nto contain event:\n\n{:?}\n\n but it wasn't found",
            events_response.events,
            expected_event
        );
    });
}

pub fn assert_vault_balance(
    mock: &MockApp,
    contract_address: &Addr,
    address: Addr,
    vault_id: Uint128,
    balance: Uint128,
) {
    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    let vault = &vault_response.vault;

    assert_eq!(
        vault.balance.amount, balance,
        "Vault balance mismatch for vault_id: {}, owner: {}",
        vault_id, address
    );
}

pub fn set_fin_price(
    deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    price: &'static Decimal,
    offer_size: &'static Uint128,
    depth: &'static Uint128,
) {
    deps.querier.update_wasm(|query| match query.clone() {
        WasmQuery::Smart { msg, .. } => match from_binary(&msg).unwrap() {
            FinQueryMsg::Book { offset, .. } => SystemResult::Ok(ContractResult::Ok(
                to_binary(&FinBookResponse {
                    base: match offset {
                        Some(0) | None => (0..depth.u128())
                            .map(|order| FinPoolResponseWithoutDenom {
                                quote_price: price.clone()
                                    + Decimal::percent(order.try_into().unwrap()),
                                total_offer_amount: offer_size.clone(),
                            })
                            .collect(),
                        _ => vec![],
                    },
                    quote: match offset {
                        Some(0) | None => (0..depth.u128())
                            .map(|order| FinPoolResponseWithoutDenom {
                                quote_price: price.clone()
                                    - Decimal::percent(order.try_into().unwrap()),
                                total_offer_amount: offer_size.clone(),
                            })
                            .collect(),
                        _ => vec![],
                    },
                })
                .unwrap(),
            )),
            _ => panic!(),
        },
        _ => panic!(),
    });
}
