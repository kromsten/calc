use super::mocks::{DENOM_UKUJI, DENOM_UUSK, USER, VALIDATOR};
use crate::{
    constants::{ONE, TEN},
    state::{
        old_pairs::PAIRS,
        old_triggers::save_old_trigger,
        old_vaults::{get_old_vault, update_old_vault},
    },
    types::{dca_plus_config::DcaPlusConfig, old_vault::OldVault},
};
use base::{
    pair::OldPair,
    triggers::trigger::{OldTimeInterval, OldTrigger, OldTriggerConfiguration},
    vaults::vault::{OldDestination, OldVaultStatus, PostExecutionAction},
};
use cosmwasm_std::{Addr, Coin, Decimal, DepsMut, Env, Timestamp, Uint128};

impl Default for OldVault {
    fn default() -> Self {
        Self {
            id: Uint128::zero(),
            created_at: Timestamp::default(),
            owner: Addr::unchecked(USER),
            label: Some("vault".to_string()),
            destinations: vec![OldDestination {
                address: Addr::unchecked(VALIDATOR),
                allocation: Decimal::percent(100),
                action: PostExecutionAction::ZDelegate,
            }],
            status: OldVaultStatus::Active,
            balance: Coin::new(TEN.into(), DENOM_UKUJI),
            swap_amount: ONE,
            pair: OldPair {
                address: Addr::unchecked("pair"),
                base_denom: DENOM_UKUJI.to_string(),
                quote_denom: DENOM_UUSK.to_string(),
            },
            slippage_tolerance: None,
            minimum_receive_amount: None,
            time_interval: OldTimeInterval::Daily,
            started_at: None,
            swapped_amount: Coin::new(0, DENOM_UKUJI),
            received_amount: Coin::new(0, DENOM_UUSK),
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
            standard_dca_received_amount: Coin::new(0, DENOM_UUSK),
            escrowed_balance: Coin::new(0, DENOM_UUSK),
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

    get_old_vault(deps.as_ref().storage, vault.id).unwrap()
}
