use crate::error::ContractError;
use crate::helpers::time::get_next_target_time;
use crate::helpers::validation::{
    assert_contract_is_not_paused, assert_deposited_denom_matches_send_denom,
    assert_exactly_one_asset, assert_vault_is_not_cancelled,
};
use crate::helpers::vault::get_risk_weighted_average_model_id;
use crate::state::events::create_event;
use crate::state::triggers::save_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::event::{EventBuilder, EventData};
use crate::types::swap_adjustment_strategy::SwapAdjustmentStrategy;
use crate::types::trigger::{Trigger, TriggerConfiguration};
use crate::types::vault::{Vault, VaultStatus};
use cosmwasm_std::{Addr, Env};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};
use shared::coin::add;

pub fn deposit_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    deps.api.addr_validate(address.as_str())?;
    assert_exactly_one_asset(info.funds.clone())?;

    let vault = get_vault(deps.storage, vault_id)?;

    if address != vault.owner {
        return Err(ContractError::CustomError {
            val: format!(
                "provided an incorrect owner address for vault id {}",
                vault_id
            ),
        });
    }

    assert_vault_is_not_cancelled(&vault)?;
    assert_deposited_denom_matches_send_denom(
        info.funds[0].denom.clone(),
        vault.balance.denom.clone(),
    )?;

    let vault_was_inactive = vault.is_inactive();
    let new_balance = add(vault.balance.clone(), info.funds[0].clone())?;

    let vault = update_vault(
        deps.storage,
        Vault {
            balance: new_balance.clone(),
            deposited_amount: add(vault.deposited_amount.clone(), info.funds[0].clone())?,
            status: if vault.is_inactive() {
                VaultStatus::Active
            } else {
                vault.status
            },
            swap_adjustment_strategy: vault.swap_adjustment_strategy.clone().map(
                |swap_adjustment_strategy| match swap_adjustment_strategy {
                    SwapAdjustmentStrategy::RiskWeightedAverage {
                        base_denom,
                        position_type,
                        ..
                    } => SwapAdjustmentStrategy::RiskWeightedAverage {
                        model_id: get_risk_weighted_average_model_id(
                            &env.block.time,
                            &new_balance,
                            &vault.swap_amount,
                            &vault.time_interval,
                        ),
                        base_denom,
                        position_type,
                    },
                    _ => swap_adjustment_strategy,
                },
            ),
            ..vault
        },
    )?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.clone(),
            EventData::DcaVaultFundsDeposited {
                amount: info.funds[0].clone(),
            },
        ),
    )?;

    if vault.is_active() && vault_was_inactive && vault.trigger.is_none() {
        save_trigger(
            deps.storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Time {
                    target_time: get_next_target_time(
                        env.block.time,
                        vault.started_at.unwrap_or(env.block.time),
                        vault.time_interval,
                    ),
                },
            },
        )?;
    };

    Ok(Response::new()
        .add_attribute("deposit", "true")
        .add_attribute("vault_id", vault.id)
        .add_attribute("owner", vault.owner)
        .add_attribute("deposited_amount", info.funds[0].amount))
}

#[cfg(test)]
mod dposit_tests {
    use super::*;
    use crate::constants::{ONE, ONE_HUNDRED, TEN};
    use crate::handlers::get_events_by_resource_id::get_events_by_resource_id_handler;
    use crate::handlers::get_vault::get_vault_handler;
    use crate::state::config::{get_config, update_config};
    use crate::tests::helpers::{instantiate_contract, setup_vault};
    use crate::tests::mocks::{ADMIN, DENOM_UKUJI, DENOM_UUSK, USER};
    use crate::types::config::Config;
    use crate::types::event::{EventBuilder, EventData};
    use crate::types::position_type::PositionType;
    use crate::types::swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy};
    use crate::types::vault::{Vault, VaultStatus};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Coin};
    use shared::coin::subtract;

    #[test]
    fn updates_the_vault_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount.clone()]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                balance: Coin::new(0, DENOM_UKUJI),
                ..Vault::default()
            },
        );

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(
            vault.balance,
            subtract(&deposit_amount, &deposit_amount).unwrap()
        );
        assert_eq!(updated_vault.balance, deposit_amount);
    }

    #[test]
    fn publishes_deposit_event() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount.clone()]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        deposit_handler(deps.as_mut(), env.clone(), info, vault.owner, vault.id).unwrap();

        let events = get_events_by_resource_id_handler(deps.as_ref(), vault.id, None, None, None)
            .unwrap()
            .events;

        assert!(events.contains(
            &EventBuilder::new(
                vault.id,
                env.block,
                EventData::DcaVaultFundsDeposited {
                    amount: deposit_amount,
                },
            )
            .build(1)
        ))
    }

    #[test]
    fn updates_inactive_vault_to_active() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Inactive,
                ..Vault::default()
            },
        );

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(vault.status, VaultStatus::Inactive);
        assert_eq!(updated_vault.status, VaultStatus::Active);
    }

    #[test]
    fn leaves_scheduled_vault_scheduled() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Scheduled,
                ..Vault::default()
            },
        );

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(vault.status, VaultStatus::Scheduled);
        assert_eq!(updated_vault.status, VaultStatus::Scheduled);
    }

    #[test]
    fn leaves_active_vault_active() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(vault.status, VaultStatus::Active);
        assert_eq!(updated_vault.status, VaultStatus::Active);
    }

    #[test]
    fn does_not_execute_trigger_for_active_vault() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let response =
            deposit_handler(deps.as_mut(), env, info, Addr::unchecked(USER), vault.id).unwrap();

        assert!(response.messages.is_empty())
    }

    #[test]
    fn does_not_execute_trigger_for_scheduled_vault() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Scheduled,
                ..Vault::default()
            },
        );

        let response =
            deposit_handler(deps.as_mut(), env, info, Addr::unchecked(USER), vault.id).unwrap();

        assert!(response.messages.is_empty())
    }

    #[test]
    fn for_cancelled_vault_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Cancelled,
                ..Vault::default()
            },
        );

        let err = deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap_err();

        assert_eq!(err.to_string(), "Error: vault is already cancelled");
    }

    #[test]
    fn with_incorrect_owner_address_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Cancelled,
                ..Vault::default()
            },
        );

        let err = deposit_handler(
            deps.as_mut(),
            env,
            info,
            Addr::unchecked("not-the-owner"),
            vault.id,
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: provided an incorrect owner address for vault id 0"
        );
    }

    #[test]
    fn with_incorrect_denom_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UUSK);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info);

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let err = deposit_handler(
            deps.as_mut(),
            env,
            mock_info(
                USER,
                &[Coin::new(ONE.into(), vault.received_amount.denom.clone())],
            ),
            vault.owner.clone(),
            vault.id,
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Error: received asset with denom {}, but needed {}",
                vault.target_denom,
                vault.get_swap_denom()
            )
        );
    }

    #[test]
    fn with_multiple_assets_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UUSK);
        let info = mock_info(ADMIN, &[deposit_amount, Coin::new(TEN.into(), DENOM_UKUJI)]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let err = deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: received 2 denoms but required exactly 1"
        );
    }

    #[test]
    fn when_contract_is_paused_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(TEN.into(), DENOM_UUSK);
        let info = mock_info(ADMIN, &[deposit_amount, Coin::new(TEN.into(), DENOM_UKUJI)]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let config = get_config(deps.as_ref().storage).unwrap();

        update_config(
            deps.as_mut().storage,
            Config {
                paused: true,
                ..config
            },
        )
        .unwrap();

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let err = deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap_err();

        assert_eq!(err.to_string(), "Error: contract is paused");
    }

    #[test]
    fn with_risk_weighted_average_strategy_should_update_model_id() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(ONE_HUNDRED.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                swap_adjustment_strategy: Some(SwapAdjustmentStrategy::RiskWeightedAverage {
                    model_id: 30,
                    base_denom: BaseDenom::Bitcoin,
                    position_type: PositionType::Enter,
                }),
                ..Vault::default()
            },
        );

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(
            vault
                .swap_adjustment_strategy
                .map(|strategy| match strategy {
                    SwapAdjustmentStrategy::RiskWeightedAverage { model_id, .. } => model_id,
                    _ => panic!("unexpected swap adjustment strategy"),
                }),
            Some(30)
        );
        assert_eq!(
            updated_vault
                .swap_adjustment_strategy
                .map(|strategy| match strategy {
                    SwapAdjustmentStrategy::RiskWeightedAverage { model_id, .. } => model_id,
                    _ => panic!("unexpected swap adjustment strategy"),
                }),
            Some(80)
        );
    }

    #[test]
    fn should_update_total_deposit() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let deposit_amount = Coin::new(ONE_HUNDRED.into(), DENOM_UKUJI);
        let info = mock_info(ADMIN, &[deposit_amount.clone()]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        deposit_handler(deps.as_mut(), env, info, vault.owner, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(vault.deposited_amount, vault.balance);
        assert_eq!(
            updated_vault.deposited_amount,
            add(vault.balance, deposit_amount).unwrap()
        );
    }
}
