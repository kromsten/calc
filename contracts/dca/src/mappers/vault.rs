use super::destination::destination_from;
use crate::types::{
    old_vault::OldVault,
    pair::Pair,
    performance_assessment_strategy::PerformanceAssessmentStrategy,
    position_type::PositionType,
    swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
    time_interval::TimeInterval,
    trigger::{Trigger, TriggerConfiguration},
    vault::{Vault, VaultStatus},
};
use base::{
    pair::OldPair,
    triggers::trigger::{OldTimeInterval, OldTrigger, OldTriggerConfiguration},
    vaults::vault::OldVaultStatus,
};
use cosmwasm_std::{Coin, Decimal, Env};
use fin_helpers::position_type::OldPositionType;
use std::str::FromStr;

pub fn vault_from(env: Env, old_vault: OldVault) -> Vault {
    Vault {
        id: old_vault.id,
        created_at: old_vault.created_at,
        owner: old_vault.owner.clone(),
        label: old_vault.label.clone(),
        destinations: old_vault
            .destinations
            .iter()
            .map(|d| destination_from(d, old_vault.owner.clone(), env.contract.address.clone()))
            .collect(),
        status: old_vault.status.clone().into(),
        balance: old_vault.balance.clone(),
        target_denom: old_vault.get_receive_denom(),
        swap_amount: old_vault.swap_amount,
        slippage_tolerance: old_vault.slippage_tolerance.unwrap_or(Decimal::percent(10)),
        minimum_receive_amount: old_vault.minimum_receive_amount,
        time_interval: old_vault.time_interval.clone().into(),
        started_at: old_vault.started_at,
        escrow_level: old_vault
            .dca_plus_config
            .clone()
            .map_or(Decimal::zero(), |dca_plus_config| {
                dca_plus_config.escrow_level
            }),
        escrowed_amount: old_vault.dca_plus_config.clone().map_or(
            Coin::new(0, old_vault.get_receive_denom()),
            |dca_plus_config| dca_plus_config.escrowed_balance,
        ),
        swapped_amount: old_vault.swapped_amount.clone(),
        deposited_amount: old_vault
            .dca_plus_config
            .clone()
            .map_or(old_vault.balance.clone(), |dca_plus_config| {
                dca_plus_config.total_deposit
            }),
        received_amount: old_vault.received_amount.clone(),
        trigger: old_vault.trigger.clone().map(|t| t.into()),
        swap_adjustment_strategy: old_vault.dca_plus_config.clone().map(|dca_plus_config| {
            SwapAdjustmentStrategy::RiskWeightedAverage {
                model_id: dca_plus_config.model_id,
                base_denom: BaseDenom::Bitcoin,
                position_type: old_vault.get_position_type().into(),
            }
        }),
        performance_assessment_strategy: old_vault.dca_plus_config.map(|dca_plus_config| {
            PerformanceAssessmentStrategy::CompareToStandardDca {
                swapped_amount: dca_plus_config.standard_dca_swapped_amount,
                received_amount: dca_plus_config.standard_dca_received_amount,
            }
        }),
    }
}

impl From<OldVaultStatus> for VaultStatus {
    fn from(old_vault_status: OldVaultStatus) -> Self {
        match old_vault_status {
            OldVaultStatus::Active => VaultStatus::Active,
            OldVaultStatus::Inactive => VaultStatus::Inactive,
            OldVaultStatus::Scheduled => VaultStatus::Scheduled,
            OldVaultStatus::Cancelled => VaultStatus::Cancelled,
        }
    }
}

impl From<OldTimeInterval> for TimeInterval {
    fn from(old_time_interval: OldTimeInterval) -> Self {
        match old_time_interval {
            OldTimeInterval::Daily => TimeInterval::Daily,
            OldTimeInterval::Weekly => TimeInterval::Weekly,
            OldTimeInterval::Monthly => TimeInterval::Monthly,
            OldTimeInterval::EverySecond => TimeInterval::EveryBlock,
            OldTimeInterval::EveryMinute => TimeInterval::EveryMinute,
            OldTimeInterval::HalfHourly => TimeInterval::HalfHourly,
            OldTimeInterval::Hourly => TimeInterval::Hourly,
            OldTimeInterval::HalfDaily => TimeInterval::HalfDaily,
            OldTimeInterval::Fortnightly => TimeInterval::Fortnightly,
        }
    }
}

impl From<OldPositionType> for PositionType {
    fn from(old_position_type: OldPositionType) -> Self {
        match old_position_type {
            OldPositionType::Enter => PositionType::Enter,
            OldPositionType::Exit => PositionType::Exit,
        }
    }
}

impl From<OldTriggerConfiguration> for TriggerConfiguration {
    fn from(old_trigger_configuration: OldTriggerConfiguration) -> Self {
        match old_trigger_configuration {
            OldTriggerConfiguration::FinLimitOrder {
                target_price,
                order_idx,
            } => TriggerConfiguration::Price {
                target_price: Decimal::from_str(&target_price.to_string()).unwrap(),
                order_idx,
            },
            OldTriggerConfiguration::Time { target_time } => {
                TriggerConfiguration::Time { target_time }
            }
        }
    }
}

impl From<OldTrigger> for Trigger {
    fn from(old_trigger: OldTrigger) -> Self {
        Trigger {
            vault_id: old_trigger.vault_id,
            configuration: old_trigger.configuration.into(),
        }
    }
}

impl From<OldPair> for Pair {
    fn from(old_pair: OldPair) -> Self {
        Pair {
            base_denom: old_pair.base_denom,
            quote_denom: old_pair.quote_denom,
            address: old_pair.address,
        }
    }
}

#[cfg(test)]
mod vault_from_tests {
    use super::vault_from;
    use crate::{
        msg::ExecuteMsg,
        types::{
            dca_plus_config::DcaPlusConfig,
            destination::Destination,
            old_vault::OldVault,
            performance_assessment_strategy::PerformanceAssessmentStrategy,
            position_type::PositionType,
            swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
            trigger::TriggerConfiguration,
            vault::Vault,
        },
    };
    use base::{
        triggers::trigger::OldTriggerConfiguration,
        vaults::vault::{OldDestination, PostExecutionAction},
    };
    use cosmwasm_std::{
        testing::mock_env, to_binary, Addr, Coin, Decimal, Decimal256, Timestamp, Uint128,
    };

    #[test]
    fn maps_default_vault_correctly() {
        assert_eq!(
            vault_from(mock_env(), OldVault::default()),
            Vault::default()
        )
    }

    #[test]
    fn maps_amounts_correctly() {
        let old_vault = OldVault {
            balance: Coin::new(23847234, "fdfewf".to_string()),
            swap_amount: Uint128::new(23443),
            dca_plus_config: Some(DcaPlusConfig {
                escrowed_balance: Coin::new(12312, "asdasqwx".to_string()),
                standard_dca_received_amount: Coin::new(42323, "dsaldnkx".to_string()),
                standard_dca_swapped_amount: Coin::new(12322, "oiywefi".to_string()),
                ..DcaPlusConfig::default()
            }),
            swapped_amount: Coin::new(3423423, "usyfceufwl".to_string()),
            received_amount: Coin::new(3487534, "edtftuywvq".to_string()),
            minimum_receive_amount: Some(Uint128::new(3284742)),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(vault.balance, old_vault.balance);
        assert_eq!(vault.swap_amount, old_vault.swap_amount);
        assert_eq!(vault.swapped_amount, old_vault.swapped_amount);
        assert_eq!(vault.received_amount, old_vault.received_amount);
        assert_eq!(
            vault.minimum_receive_amount,
            old_vault.minimum_receive_amount
        );
        assert_eq!(
            vault.performance_assessment_strategy,
            Some(PerformanceAssessmentStrategy::CompareToStandardDca {
                swapped_amount: old_vault
                    .dca_plus_config
                    .clone()
                    .unwrap()
                    .standard_dca_swapped_amount,
                received_amount: old_vault
                    .dca_plus_config
                    .clone()
                    .unwrap()
                    .standard_dca_received_amount
            })
        );
        assert_eq!(
            vault.escrowed_amount,
            old_vault.dca_plus_config.unwrap().escrowed_balance
        )
    }

    #[test]
    fn maps_dca_plus_config_correctly() {
        let old_vault = OldVault {
            dca_plus_config: Some(DcaPlusConfig {
                model_id: 78,
                escrow_level: Decimal::percent(2384),
                escrowed_balance: Coin::new(34672, "sdjafg".to_string()),
                total_deposit: Coin::new(23874, "sjdhadsba".to_string()),
                standard_dca_received_amount: Coin::new(42323, "dsaldnkx".to_string()),
                standard_dca_swapped_amount: Coin::new(12322, "oiywefi".to_string()),
            }),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(
            vault.performance_assessment_strategy,
            Some(PerformanceAssessmentStrategy::CompareToStandardDca {
                swapped_amount: old_vault
                    .dca_plus_config
                    .clone()
                    .unwrap()
                    .standard_dca_swapped_amount,
                received_amount: old_vault
                    .dca_plus_config
                    .clone()
                    .unwrap()
                    .standard_dca_received_amount
            })
        );
        assert_eq!(
            vault.deposited_amount,
            old_vault.dca_plus_config.clone().unwrap().total_deposit
        );
        assert_eq!(
            vault.escrowed_amount,
            old_vault.dca_plus_config.clone().unwrap().escrowed_balance
        );
        assert_eq!(
            vault.escrow_level,
            old_vault.dca_plus_config.clone().unwrap().escrow_level
        );
        assert_eq!(
            vault.swap_adjustment_strategy,
            Some(SwapAdjustmentStrategy::RiskWeightedAverage {
                model_id: old_vault.dca_plus_config.unwrap().model_id,
                base_denom: BaseDenom::Bitcoin,
                position_type: PositionType::Exit
            })
        );
    }

    #[test]
    fn maps_exit_position_type_correctly() {
        let pair = OldVault::default().pair;

        let old_vault = OldVault {
            balance: Coin::new(2384723, pair.base_denom.clone()),
            dca_plus_config: Some(DcaPlusConfig::default()),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(
            vault.swap_adjustment_strategy,
            Some(SwapAdjustmentStrategy::RiskWeightedAverage {
                model_id: old_vault.dca_plus_config.unwrap().model_id,
                base_denom: BaseDenom::Bitcoin,
                position_type: PositionType::Exit
            })
        );
    }

    #[test]
    fn maps_enter_position_type_correctly() {
        let pair = OldVault::default().pair;

        let old_vault = OldVault {
            balance: Coin::new(2384723, pair.quote_denom.clone()),
            dca_plus_config: Some(DcaPlusConfig::default()),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(
            vault.swap_adjustment_strategy,
            Some(SwapAdjustmentStrategy::RiskWeightedAverage {
                model_id: old_vault.dca_plus_config.unwrap().model_id,
                base_denom: BaseDenom::Bitcoin,
                position_type: PositionType::Enter
            })
        );
    }

    #[test]
    fn maps_destinations_correctly() {
        let old_vault = OldVault {
            destinations: vec![
                OldDestination {
                    address: Addr::unchecked("asjkdganas"),
                    allocation: Decimal::percent(837262),
                    action: PostExecutionAction::Send,
                },
                OldDestination {
                    address: Addr::unchecked("asjc,casanas"),
                    allocation: Decimal::percent(327),
                    action: PostExecutionAction::ZDelegate,
                },
            ],
            ..OldVault::default()
        };

        let env = mock_env();

        let vault = vault_from(env.clone(), old_vault.clone());

        assert_eq!(vault.destinations.len(), old_vault.destinations.len());
        assert_eq!(
            vault.destinations[0],
            Destination {
                address: old_vault.destinations[0].address.clone(),
                allocation: old_vault.destinations[0].allocation,
                msg: None,
            },
        );
        assert_eq!(
            vault.destinations[1],
            Destination {
                allocation: old_vault.destinations[1].allocation,
                address: env.contract.address,
                msg: Some(
                    to_binary(&ExecuteMsg::ZDelegate {
                        delegator_address: old_vault.owner,
                        validator_address: old_vault.destinations[1].address.clone()
                    })
                    .unwrap()
                )
            }
        )
    }

    #[test]
    fn maps_time_trigger_correctly() {
        let old_vault = OldVault {
            trigger: Some(OldTriggerConfiguration::Time {
                target_time: Timestamp::from_seconds(3248743),
            }),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(
            match vault.trigger.unwrap() {
                TriggerConfiguration::Time { target_time } => target_time,
                _ => panic!("Wrong trigger type"),
            },
            match old_vault.trigger.unwrap() {
                OldTriggerConfiguration::Time { target_time } => target_time,
                _ => panic!("Wrong trigger type"),
            }
        )
    }

    #[test]
    fn maps_price_trigger_correctly() {
        let old_vault = OldVault {
            trigger: Some(OldTriggerConfiguration::FinLimitOrder {
                target_price: Decimal256::percent(32487632),
                order_idx: Some(Uint128::new(28347)),
            }),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(
            match vault.trigger.unwrap() {
                TriggerConfiguration::Price {
                    target_price,
                    order_idx,
                } => (target_price.into(), order_idx),
                _ => panic!("Wrong trigger type"),
            },
            match old_vault.trigger.unwrap() {
                OldTriggerConfiguration::FinLimitOrder {
                    target_price,
                    order_idx,
                } => (target_price, order_idx),
                _ => panic!("Wrong trigger type"),
            }
        )
    }

    #[test]
    fn maps_target_base_denom_correctly() {
        let pair = OldVault::default().pair;

        let old_vault = OldVault {
            balance: Coin::new(387462, pair.quote_denom.clone()),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(vault.target_denom, pair.base_denom);
    }

    #[test]
    fn maps_target_quote_denom_correctly() {
        let pair = OldVault::default().pair;

        let old_vault = OldVault {
            balance: Coin::new(387462, pair.base_denom.clone()),
            ..OldVault::default()
        };

        let vault = vault_from(mock_env(), old_vault.clone());

        assert_eq!(vault.target_denom, pair.quote_denom);
    }
}
