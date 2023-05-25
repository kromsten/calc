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
use cosmwasm_std::{Addr, Coin, Decimal};
use fin_helpers::position_type::OldPositionType;
use std::str::FromStr;

pub fn vault_from(contract_address: Addr, old_vault: OldVault) -> Vault {
    Vault {
        id: old_vault.id,
        created_at: old_vault.created_at,
        owner: old_vault.owner.clone(),
        label: old_vault.label.clone(),
        destinations: old_vault
            .destinations
            .iter()
            .map(|d| destination_from(d, old_vault.owner.clone(), contract_address.clone()))
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
            vault::VaultStatus,
        },
    };
    use base::{
        triggers::trigger::OldTriggerConfiguration,
        vaults::vault::{OldDestination, OldVaultStatus, PostExecutionAction},
    };
    use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Decimal256, Timestamp, Uint128};

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

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

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
    fn maps_time_trigger_correctly() {
        let target_time = Timestamp::from_seconds(123123);

        let old_vault = OldVault {
            trigger: Some(OldTriggerConfiguration::Time { target_time }),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(
            vault.trigger,
            Some(TriggerConfiguration::Time { target_time })
        )
    }

    #[test]
    fn maps_price_trigger_correctly() {
        let target_price_percent = 2311;
        let order_idx = Uint128::new(213123);

        let old_vault = OldVault {
            trigger: Some(OldTriggerConfiguration::FinLimitOrder {
                target_price: Decimal256::percent(target_price_percent),
                order_idx: Some(order_idx),
            }),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(
            vault.trigger,
            Some(TriggerConfiguration::Price {
                target_price: Decimal::percent(target_price_percent),
                order_idx: Some(order_idx)
            })
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

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

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

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

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

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

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

        let old_staking_router_address = Addr::unchecked("staking-router");
        let vault = vault_from(old_staking_router_address.clone(), old_vault.clone());

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
                address: old_staking_router_address,
                msg: Some(
                    to_binary(&ExecuteMsg::OldZDelegate {
                        delegator_address: old_vault.owner,
                        validator_address: old_vault.destinations[1].address.clone(),
                        amount: Uint128::zero(),
                        denom: "".to_string()
                    })
                    .unwrap()
                )
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

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.target_denom, pair.base_denom);
    }

    #[test]
    fn maps_target_quote_denom_correctly() {
        let pair = OldVault::default().pair;

        let old_vault = OldVault {
            balance: Coin::new(387462, pair.base_denom.clone()),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.target_denom, pair.quote_denom);
    }

    #[test]
    fn maps_statuses_correctly() {
        let old_vault = OldVault {
            status: OldVaultStatus::Active,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.status, VaultStatus::Active);

        let old_vault = OldVault {
            status: OldVaultStatus::Inactive,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.status, VaultStatus::Inactive);

        let old_vault = OldVault {
            status: OldVaultStatus::Cancelled,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.status, VaultStatus::Cancelled);

        let old_vault = OldVault {
            status: OldVaultStatus::Scheduled,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.status, VaultStatus::Scheduled);
    }

    #[test]
    fn maps_timestamps_correctly() {
        let old_vault = OldVault {
            created_at: Timestamp::from_seconds(2312123),
            started_at: Some(Timestamp::from_seconds(2312123)),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.created_at, old_vault.created_at);
        assert_eq!(vault.started_at, old_vault.started_at);

        let old_vault = OldVault {
            started_at: None,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.started_at, old_vault.started_at);
    }

    #[test]
    fn maps_slippage_tolerance_correctly() {
        let old_vault = OldVault {
            slippage_tolerance: Some(Decimal::percent(123)),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(
            vault.slippage_tolerance,
            old_vault.slippage_tolerance.unwrap()
        );

        let old_vault = OldVault {
            slippage_tolerance: None,
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.slippage_tolerance, Decimal::percent(10));
    }

    #[test]
    fn maps_metadata_correctly() {
        let old_vault = OldVault {
            id: Uint128::new(3742),
            label: Some("a,sjd;as".to_string()),
            owner: Addr::unchecked("adsbuybwq;idwuqn"),
            ..OldVault::default()
        };

        let vault = vault_from(Addr::unchecked("staking-router"), old_vault.clone());

        assert_eq!(vault.id, old_vault.id);
        assert_eq!(vault.label, old_vault.label);
        assert_eq!(vault.owner, old_vault.owner);
    }
}
