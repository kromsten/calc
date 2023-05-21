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
        old_triggers::TRIGGERS, pairs::save_pair, triggers::save_trigger,
    },
    types::config::Config,
};
use base::{pair::OldPair, triggers::trigger::OldTrigger};
use cosmwasm_std::{DepsMut, Order, Response, StdError};
use cw2::{get_contract_version, set_contract_version};

pub fn migrate_handler(deps: DepsMut, msg: MigrateMsg) -> Result<Response, ContractError> {
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
            default_swap_fee_percent: msg.clone().default_swap_fee_percent,
            weighted_scale_swap_fee_percent: msg.clone().weighted_scale_swap_fee_percent,
            automation_fee_percent: msg.clone().automation_fee_percent,
            default_page_limit: msg.clone().default_page_limit,
            paused: msg.clone().paused,
            risk_weighted_average_escrow_level: msg.clone().risk_weighted_average_escrow_level,
            twap_period: msg.clone().twap_period,
            default_slippage_tolerance: msg.clone().default_slippage_tolerance,
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

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("msg", format!("{:#?}", msg)))
}

#[cfg(test)]
mod migrate_tests {
    use base::{
        pair::OldPair,
        triggers::trigger::{OldTrigger, OldTriggerConfiguration},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Decimal, Decimal256, Order, Uint128,
    };

    use crate::{
        contract::migrate,
        msg::MigrateMsg,
        state::{
            old_pairs::PAIRS,
            old_triggers::save_old_trigger,
            pairs::{find_pair, get_pairs},
            triggers::{get_trigger, trigger_store},
        },
        tests::{old_helpers::instantiate_contract, old_mocks::ADMIN},
        types::{
            fee_collector::FeeCollector,
            pair::Pair,
            trigger::{Trigger, TriggerConfiguration},
        },
    };

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
                configuration: TriggerConfiguration::FinLimitOrder {
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
}
