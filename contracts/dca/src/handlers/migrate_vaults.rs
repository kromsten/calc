use crate::{
    error::ContractError,
    mappers::vault::vault_from,
    state::{
        old_vaults::get_old_vaults,
        triggers::save_trigger,
        vaults::{get_vaults, migrate_vault},
    },
    types::trigger::Trigger,
};
use cosmwasm_std::{DepsMut, Env, Response, Uint128};

pub fn migrate_vaults_handler(
    deps: DepsMut,
    env: Env,
    limit: u16,
) -> Result<Response, ContractError> {
    let start_after_vault_id = get_vaults(deps.storage, None, Some(1), Some(true))?
        .first()
        .map_or(Uint128::zero(), |vault| vault.id);

    let old_vaults_to_be_migrated =
        get_old_vaults(deps.storage, Some(start_after_vault_id), Some(limit))?;

    if old_vaults_to_be_migrated.is_empty() {
        return Ok(Response::new().add_attribute("migrated_ids", "none"));
    }

    let mut latest_migrated_vault_id = start_after_vault_id + Uint128::one();

    for old_vault in old_vaults_to_be_migrated {
        latest_migrated_vault_id = old_vault.id;

        migrate_vault(
            deps.storage,
            vault_from(env.clone().contract.address, old_vault.clone()),
        )?;

        if let Some(trigger) = old_vault.trigger {
            save_trigger(
                deps.storage,
                Trigger {
                    vault_id: old_vault.id,
                    configuration: trigger.into(),
                },
            )?;
        }
    }

    Ok(Response::new().add_attribute(
        "migrated_ids",
        format!(
            "{}-{}",
            start_after_vault_id + Uint128::one(),
            latest_migrated_vault_id
        ),
    ))
}

#[cfg(test)]
mod migrate_vaults_tests {
    use crate::{
        handlers::migrate_vaults::migrate_vaults_handler,
        mappers::vault::vault_from,
        state::{
            old_vaults::{get_old_vault, get_old_vaults},
            vaults::{get_vault, get_vaults},
        },
        tests::{
            helpers::instantiate_contract, helpers::setup_vault, mocks::ADMIN,
            old_helpers::setup_old_vault,
        },
        types::{dca_plus_config::DcaPlusConfig, old_vault::OldVault},
    };
    use base::triggers::trigger::OldTriggerConfiguration;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Decimal256, Uint128,
    };

    #[test]
    fn migrates_vaults_to_empty_map() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 1u128..11u128 {
            setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    trigger: Some(if i % 2 == 0 {
                        OldTriggerConfiguration::Time {
                            target_time: env.block.time.plus_seconds(1000).into(),
                        }
                    } else {
                        OldTriggerConfiguration::FinLimitOrder {
                            target_price: Decimal256::percent(80 + i as u64),
                            order_idx: Some(Uint128::new(i)),
                        }
                    }),
                    dca_plus_config: Some(DcaPlusConfig::default()),
                    ..OldVault::default()
                },
            );
        }

        migrate_vaults_handler(deps.as_mut(), env.clone(), 100).unwrap();

        let old_vaults = get_old_vaults(&deps.storage, None, None).unwrap();
        let vaults = get_vaults(&deps.storage, None, None, None).unwrap();

        assert_eq!(old_vaults.len(), 10);
        assert_eq!(vaults.len(), 10);

        for i in 1..11 {
            let old_vault = get_old_vault(&deps.storage, Uint128::new(i)).unwrap();
            let vault = get_vault(&deps.storage, Uint128::new(i)).unwrap();

            assert_eq!(vault_from(env.clone().contract.address, old_vault), vault);
        }
    }

    #[test]
    fn migrates_vaults_to_non_empty_map() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 1u128..51u128 {
            let old_vault = setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    trigger: Some(if i % 2 == 0 {
                        OldTriggerConfiguration::Time {
                            target_time: env.clone().block.time.plus_seconds(1000),
                        }
                    } else {
                        OldTriggerConfiguration::FinLimitOrder {
                            target_price: Decimal256::percent(80 + i as u64),
                            order_idx: Some(Uint128::new(i)),
                        }
                    }),
                    dca_plus_config: Some(DcaPlusConfig::default()),
                    ..OldVault::default()
                },
            );

            if i < 21 {
                setup_vault(
                    deps.as_mut(),
                    env.clone(),
                    vault_from(env.clone().contract.address, old_vault),
                );
            }
        }

        let old_vaults_before = get_old_vaults(&deps.storage, None, Some(100)).unwrap();
        let vaults_before = get_vaults(&deps.storage, None, Some(100), None).unwrap();

        migrate_vaults_handler(deps.as_mut(), env.clone(), 100).unwrap();

        let old_vaults_after = get_old_vaults(&deps.storage, None, Some(100)).unwrap();
        let vaults_after = get_vaults(&deps.storage, None, Some(100), None).unwrap();

        assert_eq!(old_vaults_before.len(), 50);
        assert_eq!(vaults_before.len(), 20);

        assert_eq!(old_vaults_after.len(), 50);
        assert_eq!(vaults_after.len(), 50);

        for i in 1..51 {
            let old_vault = get_old_vault(&deps.storage, Uint128::new(i)).unwrap();
            let vault = get_vault(&deps.storage, Uint128::new(i)).unwrap();

            assert_eq!(vault_from(env.clone().contract.address, old_vault), vault);
        }
    }

    #[test]
    fn respects_limit_when_migrating_vaults() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 1u128..51u128 {
            let old_vault = setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    trigger: Some(if i % 2 == 0 {
                        OldTriggerConfiguration::Time {
                            target_time: env.block.time.plus_seconds(1000).into(),
                        }
                    } else {
                        OldTriggerConfiguration::FinLimitOrder {
                            target_price: Decimal256::percent(80 + i as u64),
                            order_idx: Some(Uint128::new(i)),
                        }
                    }),
                    dca_plus_config: Some(DcaPlusConfig::default()),
                    ..OldVault::default()
                },
            );

            if i < 21 {
                setup_vault(
                    deps.as_mut(),
                    env.clone(),
                    vault_from(env.clone().contract.address, old_vault),
                );
            }
        }

        let old_vaults_before = get_old_vaults(&deps.storage, None, Some(100)).unwrap();
        let vaults_before = get_vaults(&deps.storage, None, Some(100), None).unwrap();

        migrate_vaults_handler(deps.as_mut(), env.clone(), 10).unwrap();

        let old_vaults_after = get_old_vaults(&deps.storage, None, Some(100)).unwrap();
        let vaults_after = get_vaults(&deps.storage, None, Some(100), None).unwrap();

        assert_eq!(old_vaults_before.len(), 50);
        assert_eq!(vaults_before.len(), 20);

        assert_eq!(old_vaults_after.len(), 50);
        assert_eq!(vaults_after.len(), 30);

        for i in 1..31 {
            let old_vault = get_old_vault(&deps.storage, Uint128::new(i)).unwrap();
            let vault = get_vault(&deps.storage, Uint128::new(i)).unwrap();

            assert_eq!(vault_from(env.clone().contract.address, old_vault), vault);
        }
    }

    #[test]
    fn migrates_vaults_from_empty_until_full() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 1u128..51u128 {
            setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    trigger: Some(if i % 2 == 0 {
                        OldTriggerConfiguration::Time {
                            target_time: env.block.time.plus_seconds(1000).into(),
                        }
                    } else {
                        OldTriggerConfiguration::FinLimitOrder {
                            target_price: Decimal256::percent(80 + i as u64),
                            order_idx: Some(Uint128::new(i)),
                        }
                    }),
                    dca_plus_config: Some(DcaPlusConfig::default()),
                    ..OldVault::default()
                },
            );
        }

        let limit = 10;

        for i in 1..5 {
            migrate_vaults_handler(deps.as_mut(), env.clone(), limit).unwrap();

            let migrated_vaults = get_vaults(&deps.storage, None, Some(100), None).unwrap();
            assert_eq!(migrated_vaults.len(), i * limit as usize);
        }

        migrate_vaults_handler(deps.as_mut(), env.clone(), limit).unwrap();

        let migrated_vaults = get_vaults(&deps.storage, None, Some(100), None).unwrap();
        assert_eq!(migrated_vaults.len(), 50);

        migrate_vaults_handler(deps.as_mut(), env.clone(), limit).unwrap();

        let migrated_vaults = get_vaults(&deps.storage, None, Some(100), None).unwrap();
        assert_eq!(migrated_vaults.len(), 50);

        for i in 1..51 {
            let vault_id = Uint128::new(i);

            let old_vault = get_old_vault(deps.as_ref().storage, vault_id).unwrap();
            let vault = get_vault(deps.as_ref().storage, vault_id).unwrap();

            assert_eq!(vault_from(env.clone().contract.address, old_vault), vault);
        }
    }
}
