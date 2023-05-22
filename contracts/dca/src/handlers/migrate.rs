use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    helpers::validation::{
        assert_addresses_are_valid, assert_fee_collector_addresses_are_valid,
        assert_fee_collector_allocations_add_up_to_one, assert_fee_level_is_valid,
        assert_no_more_than_10_fee_collectors, assert_page_limit_is_valid,
        assert_risk_weighted_average_escrow_level_is_no_greater_than_100_percent,
        assert_slippage_tolerance_is_less_than_or_equal_to_one, assert_twap_period_is_valid,
    },
    msg::MigrateMsg,
    state::{
        config::update_config, old_config::get_old_config, old_pairs::PAIRS,
        old_swap_adjustments::get_old_swap_adjustment, old_triggers::TRIGGERS, pairs::save_pair,
        swap_adjustments::update_swap_adjustment, triggers::save_trigger,
    },
    types::{
        config::Config,
        swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
    },
};
use base::{pair::OldPair, triggers::trigger::OldTrigger};
use cosmwasm_std::{DepsMut, Env, Order, Response, StdError};
use cw2::{get_contract_version, set_contract_version};
use fin_helpers::position_type::OldPositionType;

pub fn migrate_handler(
    deps: DepsMut,
    env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    if contract_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }

    #[allow(clippy::cmp_owned)]
    if contract_version.version > CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    assert_fee_level_is_valid(&msg.default_swap_fee_percent)?;
    assert_fee_level_is_valid(&msg.weighted_scale_swap_fee_percent)?;
    assert_fee_level_is_valid(&msg.automation_fee_percent)?;
    assert_page_limit_is_valid(Some(msg.default_page_limit))?;
    assert_slippage_tolerance_is_less_than_or_equal_to_one(msg.default_slippage_tolerance)?;
    assert_twap_period_is_valid(msg.twap_period)?;
    assert_addresses_are_valid(deps.as_ref(), &msg.executors, "executor")?;
    assert_no_more_than_10_fee_collectors(&msg.fee_collectors)?;
    assert_fee_collector_addresses_are_valid(deps.as_ref(), &msg.fee_collectors)?;
    assert_fee_collector_allocations_add_up_to_one(&msg.fee_collectors)?;
    assert_risk_weighted_average_escrow_level_is_no_greater_than_100_percent(
        msg.risk_weighted_average_escrow_level,
    )?;

    let old_config = get_old_config(deps.storage)?;

    update_config(
        deps.storage,
        Config {
            executors: msg.clone().executors,
            fee_collectors: msg.clone().fee_collectors,
            default_swap_fee_percent: msg.default_swap_fee_percent,
            weighted_scale_swap_fee_percent: msg.weighted_scale_swap_fee_percent,
            automation_fee_percent: msg.automation_fee_percent,
            default_page_limit: msg.default_page_limit,
            paused: msg.paused,
            risk_weighted_average_escrow_level: msg.risk_weighted_average_escrow_level,
            twap_period: msg.twap_period,
            default_slippage_tolerance: msg.default_slippage_tolerance,
            admin: old_config.admin,
        },
    )?;

    let old_pairs = PAIRS
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|result| result.map(|(_, pair)| pair))
        .collect::<Vec<OldPair>>();

    for old_pair in old_pairs {
        save_pair(deps.storage, &old_pair.into())?;
    }

    let old_triggers = TRIGGERS
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|result| result.map(|(_, trigger)| trigger))
        .collect::<Vec<OldTrigger>>();

    for old_trigger in old_triggers {
        save_trigger(deps.storage, old_trigger.into())?;
    }

    for model_id in [30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90] {
        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            let swap_adjustment = get_old_swap_adjustment(
                deps.storage,
                position_type.clone(),
                model_id,
                env.block.time,
            )?;

            update_swap_adjustment(
                deps.storage,
                SwapAdjustmentStrategy::RiskWeightedAverage {
                    model_id,
                    base_denom: BaseDenom::Bitcoin,
                    position_type: position_type.into(),
                },
                swap_adjustment,
                env.block.time,
            )?;
        }
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("msg", format!("{:#?}", msg)))
}

#[cfg(test)]
mod migrate_tests {
    use crate::{
        contract::migrate,
        msg::MigrateMsg,
        state::{
            old_pairs::PAIRS,
            old_swap_adjustments::update_swap_adjustments,
            old_triggers::save_old_trigger,
            pairs::{find_pair, get_pairs},
            swap_adjustments::get_swap_adjustment,
            triggers::{get_trigger, trigger_store},
        },
        tests::{helpers::instantiate_contract, mocks::ADMIN},
        types::{
            fee_collector::FeeCollector,
            pair::Pair,
            position_type::PositionType,
            swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
            trigger::{Trigger, TriggerConfiguration},
        },
    };
    use base::{
        pair::OldPair,
        triggers::trigger::{OldTrigger, OldTriggerConfiguration},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Decimal, Decimal256, Order, Uint128,
    };
    use fin_helpers::position_type::OldPositionType;

    #[test]
    fn migrates_pair() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let pair = OldPair {
                base_denom: format!("base-{}", i),
                quote_denom: format!("quote-{}", i),
                address: Addr::unchecked(format!("pair-{}", i)),
            };

            PAIRS
                .save(deps.as_mut().storage, pair.address.clone(), &pair)
                .unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(120)),
                    (35, Decimal::percent(120)),
                    (40, Decimal::percent(120)),
                    (45, Decimal::percent(120)),
                    (50, Decimal::percent(120)),
                    (55, Decimal::percent(120)),
                    (60, Decimal::percent(120)),
                    (65, Decimal::percent(120)),
                    (70, Decimal::percent(120)),
                    (75, Decimal::percent(120)),
                    (80, Decimal::percent(120)),
                    (85, Decimal::percent(120)),
                    (90, Decimal::percent(120)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        let pair = find_pair(
            deps.as_ref().storage,
            ["base-11".to_string(), "quote-11".to_string()],
        )
        .unwrap();

        assert_eq!(
            pair,
            Pair {
                base_denom: "base-11".to_string(),
                quote_denom: "quote-11".to_string(),
                address: Addr::unchecked("pair-11"),
            }
        );
    }

    #[test]
    fn migrates_all_pairs() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let pair = OldPair {
                base_denom: format!("base-{}", i),
                quote_denom: format!("quote-{}", i),
                address: Addr::unchecked(format!("pair-{}", i)),
            };

            PAIRS
                .save(deps.as_mut().storage, pair.address.clone(), &pair)
                .unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(120)),
                    (35, Decimal::percent(120)),
                    (40, Decimal::percent(120)),
                    (45, Decimal::percent(120)),
                    (50, Decimal::percent(120)),
                    (55, Decimal::percent(120)),
                    (60, Decimal::percent(120)),
                    (65, Decimal::percent(120)),
                    (70, Decimal::percent(120)),
                    (75, Decimal::percent(120)),
                    (80, Decimal::percent(120)),
                    (85, Decimal::percent(120)),
                    (90, Decimal::percent(120)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        let pairs = get_pairs(deps.as_ref().storage);

        assert_eq!(pairs.len(), 20);
    }

    #[test]
    fn migrates_time_trigger() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let trigger = OldTrigger {
                vault_id: Uint128::new(i),
                configuration: OldTriggerConfiguration::Time {
                    target_time: env.block.time.plus_seconds(i as u64),
                },
            };

            save_old_trigger(deps.as_mut().storage, trigger).unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(120)),
                    (35, Decimal::percent(120)),
                    (40, Decimal::percent(120)),
                    (45, Decimal::percent(120)),
                    (50, Decimal::percent(120)),
                    (55, Decimal::percent(120)),
                    (60, Decimal::percent(120)),
                    (65, Decimal::percent(120)),
                    (70, Decimal::percent(120)),
                    (75, Decimal::percent(120)),
                    (80, Decimal::percent(120)),
                    (85, Decimal::percent(120)),
                    (90, Decimal::percent(120)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        let trigger = get_trigger(deps.as_ref().storage, Uint128::new(10)).unwrap();

        assert_eq!(
            trigger,
            Some(Trigger {
                vault_id: Uint128::new(10),
                configuration: TriggerConfiguration::Time {
                    target_time: env.block.time.plus_seconds(10),
                },
            })
        );
    }

    #[test]
    fn migrates_price_trigger() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let trigger = OldTrigger {
                vault_id: Uint128::new(i),
                configuration: OldTriggerConfiguration::FinLimitOrder {
                    target_price: Decimal256::percent(132),
                    order_idx: Some(Uint128::new(i)),
                },
            };

            save_old_trigger(deps.as_mut().storage, trigger).unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(120)),
                    (35, Decimal::percent(120)),
                    (40, Decimal::percent(120)),
                    (45, Decimal::percent(120)),
                    (50, Decimal::percent(120)),
                    (55, Decimal::percent(120)),
                    (60, Decimal::percent(120)),
                    (65, Decimal::percent(120)),
                    (70, Decimal::percent(120)),
                    (75, Decimal::percent(120)),
                    (80, Decimal::percent(120)),
                    (85, Decimal::percent(120)),
                    (90, Decimal::percent(120)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        let trigger = get_trigger(deps.as_ref().storage, Uint128::new(10)).unwrap();

        assert_eq!(
            trigger,
            Some(Trigger {
                vault_id: Uint128::new(10),
                configuration: TriggerConfiguration::Price {
                    target_price: Decimal::percent(132),
                    order_idx: Some(Uint128::new(10)),
                },
            })
        );
    }

    #[test]
    fn migrates_all_triggers() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let trigger = OldTrigger {
                vault_id: Uint128::new(i),
                configuration: OldTriggerConfiguration::FinLimitOrder {
                    target_price: Decimal256::percent(132),
                    order_idx: Some(Uint128::new(i)),
                },
            };

            save_old_trigger(deps.as_mut().storage, trigger).unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(120)),
                    (35, Decimal::percent(120)),
                    (40, Decimal::percent(120)),
                    (45, Decimal::percent(120)),
                    (50, Decimal::percent(120)),
                    (55, Decimal::percent(120)),
                    (60, Decimal::percent(120)),
                    (65, Decimal::percent(120)),
                    (70, Decimal::percent(120)),
                    (75, Decimal::percent(120)),
                    (80, Decimal::percent(120)),
                    (85, Decimal::percent(120)),
                    (90, Decimal::percent(120)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        assert_eq!(
            trigger_store()
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .count(),
            20
        );
    }

    #[test]
    fn migrates_all_swap_adjustments() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0..20 {
            let trigger = OldTrigger {
                vault_id: Uint128::new(i),
                configuration: OldTriggerConfiguration::FinLimitOrder {
                    target_price: Decimal256::percent(132),
                    order_idx: Some(Uint128::new(i)),
                },
            };

            save_old_trigger(deps.as_mut().storage, trigger).unwrap();
        }

        for position_type in [OldPositionType::Enter, OldPositionType::Exit] {
            update_swap_adjustments(
                deps.as_mut().storage,
                position_type,
                vec![
                    (30, Decimal::percent(130)),
                    (35, Decimal::percent(135)),
                    (40, Decimal::percent(140)),
                    (45, Decimal::percent(145)),
                    (50, Decimal::percent(150)),
                    (55, Decimal::percent(155)),
                    (60, Decimal::percent(160)),
                    (65, Decimal::percent(165)),
                    (70, Decimal::percent(170)),
                    (75, Decimal::percent(175)),
                    (80, Decimal::percent(180)),
                    (85, Decimal::percent(185)),
                    (90, Decimal::percent(190)),
                ],
                env.block.time,
            )
            .unwrap();
        }

        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                executors: vec![Addr::unchecked(ADMIN)],
                fee_collectors: vec![FeeCollector {
                    allocation: Decimal::percent(100),
                    address: ADMIN.to_string(),
                }],
                default_swap_fee_percent: Decimal::percent(2),
                weighted_scale_swap_fee_percent: Decimal::percent(2),
                automation_fee_percent: Decimal::percent(2),
                default_page_limit: 30,
                paused: true,
                risk_weighted_average_escrow_level: Decimal::percent(5),
                twap_period: 60,
                default_slippage_tolerance: Decimal::percent(10),
            },
        )
        .unwrap();

        for model_id in [30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90] {
            for position_type in [PositionType::Enter, PositionType::Exit] {
                let swap_adjustment = get_swap_adjustment(
                    &deps.storage,
                    SwapAdjustmentStrategy::RiskWeightedAverage {
                        model_id,
                        base_denom: BaseDenom::Bitcoin,
                        position_type,
                    },
                    env.block.time,
                );

                assert_eq!(swap_adjustment, Decimal::percent(100 + (model_id as u64)));
            }
        }
    }
}
