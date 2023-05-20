use std::str::FromStr;

use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Decimal,
};
use cw_multi_test::Executor;

use crate::{
    handlers::update_config::update_config_handler,
    msg::{ConfigResponse, ExecuteMsg, QueryMsg},
    state::config::{get_config, FeeCollector},
    tests::helpers::instantiate_contract,
};

use super::mocks::{fin_contract_unfilled_limit_order, MockApp, ADMIN};

#[test]
fn update_executors_with_no_value_should_not_change_value() {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN, &[]);

    instantiate_contract(deps.as_mut(), mock_env(), info.clone());

    let config_before_update = get_config(deps.as_ref().storage).unwrap();

    update_config_handler(
        deps.as_mut(),
        info,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let config_after_update = get_config(deps.as_ref().storage).unwrap();

    assert_eq!(
        config_after_update.executors,
        config_before_update.executors
    );
}

#[test]
fn update_executors_with_valid_value_should_succeed() {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN, &[]);

    instantiate_contract(deps.as_mut(), mock_env(), info.clone());

    let executors = Some(vec![
        Addr::unchecked("executor-1"),
        Addr::unchecked("executor-2"),
    ]);

    update_config_handler(
        deps.as_mut(),
        info,
        executors.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let config = get_config(deps.as_ref().storage).unwrap();

    assert_eq!(config.executors, executors.unwrap());
}

#[test]
fn update_fee_percent_with_valid_value_should_succeed() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: Some(vec![FeeCollector {
                    address: ADMIN.to_string(),
                    allocation: Decimal::from_str("1").unwrap(),
                }]),
                swap_fee_percent: Some(Decimal::from_str("0.015").unwrap()),
                delegation_fee_percent: Some(Decimal::from_str("0.0075").unwrap()),
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap();
}

#[test]
fn update_swap_fee_percent_more_than_100_percent_should_fail() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    let error = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: Some(vec![FeeCollector {
                    address: ADMIN.to_string(),
                    allocation: Decimal::from_str("1").unwrap(),
                }]),
                swap_fee_percent: Some(Decimal::from_str("1.5").unwrap()),
                delegation_fee_percent: Some(Decimal::from_str("0.0075").unwrap()),
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        error.root_cause().to_string(),
        "Generic error: swap_fee_percent must be less than 100%, and expressed as a ratio out of 1 (i.e. use 0.015 to represent a fee of 1.5%)"
    )
}

#[test]
fn update_fee_collectors_with_no_value_should_succeed() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: None,
                swap_fee_percent: Some(Decimal::from_str("0.015").unwrap()),
                delegation_fee_percent: Some(Decimal::from_str("0.0075").unwrap()),
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap();
}

#[test]
fn update_fee_collectors_with_valid_value_should_succeed() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: Some(vec![
                    FeeCollector {
                        address: ADMIN.to_string(),
                        allocation: Decimal::from_str("0.9").unwrap(),
                    },
                    FeeCollector {
                        address: ADMIN.to_string(),
                        allocation: Decimal::from_str("0.1").unwrap(),
                    },
                ]),
                swap_fee_percent: None,
                delegation_fee_percent: None,
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap();
}

#[test]
fn update_fee_collectors_with_total_allocations_more_than_100_percent_should_fail() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    let error = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: Some(vec![
                    FeeCollector {
                        address: ADMIN.to_string(),
                        allocation: Decimal::from_str("1").unwrap(),
                    },
                    FeeCollector {
                        address: ADMIN.to_string(),
                        allocation: Decimal::from_str("1").unwrap(),
                    },
                ]),
                swap_fee_percent: None,
                delegation_fee_percent: None,
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        error.root_cause().to_string(),
        "Error: fee collector allocations must add up to 1"
    )
}

#[test]
fn update_dca_plus_escrow_level_with_valid_value_should_succeed() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: None,
                swap_fee_percent: None,
                delegation_fee_percent: None,
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: Some(Decimal::percent(19)),
            },
            &[],
        )
        .unwrap();

    let config_response = mock
        .app
        .wrap()
        .query_wasm_smart::<ConfigResponse>(
            mock.dca_contract_address.clone(),
            &QueryMsg::GetConfig {},
        )
        .unwrap();

    assert_eq!(
        config_response.config.risk_weighted_average_escrow_level,
        Decimal::percent(19)
    );
}

#[test]
fn update_dca_plus_escrow_level_more_than_100_percent_should_fail() {
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order());

    let error = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: None,
                swap_fee_percent: None,
                delegation_fee_percent: None,
                staking_router_address: None,
                page_limit: None,
                paused: None,
                dca_plus_escrow_level: Some(Decimal::percent(150)),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        error.root_cause().to_string(),
        "Error: dca_plus_escrow_level cannot be greater than 100%"
    )
}
