use super::destination::destination_from;
use crate::types::{
    old_vault::OldVault,
    performance_assessment_strategy::PerformanceAssessmentStrategy,
    position_type::PositionType,
    swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
    time_interval::TimeInterval,
    trigger::TriggerConfiguration,
    vault::{Vault, VaultStatus},
};
use base::{
    triggers::trigger::{OldTimeInterval, OldTriggerConfiguration},
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
            Coin::new(0, old_vault.balance.denom.clone()),
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
            OldTimeInterval::EverySecond => TimeInterval::EverySecond,
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
            } => TriggerConfiguration::FinLimitOrder {
                target_price: Decimal::from_str(&target_price.to_string()).unwrap(),
                order_idx,
            },
            OldTriggerConfiguration::Time { target_time } => {
                TriggerConfiguration::Time { target_time }
            }
        }
    }
}
