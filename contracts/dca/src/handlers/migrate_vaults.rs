use crate::{
    error::ContractError,
    mappers::vault::vault_from,
    state::{
        old_vaults::get_old_vaults,
        vaults::{get_vaults, migrate_vault},
    },
};
use cosmwasm_std::{DepsMut, Env, Response, Uint128};

pub fn migrate_vaults_handler(
    deps: DepsMut,
    env: Env,
    limit: u16,
) -> Result<Response, ContractError> {
    let latest_migrated_vault_id = get_vaults(deps.storage, None, Some(1), Some(true))?
        .first()
        .map(|vault| vault.id);

    let old_vaults_to_be_migrated =
        get_old_vaults(deps.storage, latest_migrated_vault_id, Some(limit))?;

    for old_vault in old_vaults_to_be_migrated {
        migrate_vault(deps.storage, vault_from(env.clone(), old_vault))?;
    }

    Ok(Response::new().add_attribute(
        "migrated_ids",
        format!(
            "{}-{}",
            latest_migrated_vault_id.map_or(Uint128::zero(), |id| id + Uint128::one()),
            latest_migrated_vault_id
                .map_or(Uint128::zero(), |id| id + Uint128::new((limit + 1).into()))
        ),
    ))
}

#[cfg(test)]
mod migrate_vaults_tests {
    use crate::{
        handlers::migrate_vaults::migrate_vaults_handler,
        state::{old_vaults::get_old_vaults, vaults::get_vaults},
        tests::{
            helpers::setup_vault,
            old_helpers::{instantiate_contract, setup_old_vault},
            old_mocks::ADMIN,
        },
        types::{old_vault::OldVault, vault::Vault},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Uint128,
    };

    #[test]
    fn migrates_vaults_to_empty_map() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0u128..10u128 {
            setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    ..OldVault::default()
                },
            );
        }

        migrate_vaults_handler(deps.as_mut(), env.clone(), 100).unwrap();

        let old_vaults = get_old_vaults(&deps.storage, None, None).unwrap();
        let vaults = get_vaults(&deps.storage, None, None, None).unwrap();

        assert_eq!(old_vaults.len(), 10);
        assert_eq!(vaults.len(), 10);
    }

    #[test]
    fn migrates_vaults_to_non_empty_map() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0u128..50u128 {
            setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    ..OldVault::default()
                },
            );

            if i < 20 {
                setup_vault(
                    deps.as_mut(),
                    env.clone(),
                    Vault {
                        id: Uint128::new(i),
                        ..Vault::default()
                    },
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
    }

    #[test]
    fn respects_limit_when_migrating_vaults() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

        for i in 0u128..50u128 {
            setup_old_vault(
                deps.as_mut(),
                env.clone(),
                OldVault {
                    id: Uint128::new(i),
                    ..OldVault::default()
                },
            );

            if i < 20 {
                setup_vault(
                    deps.as_mut(),
                    env.clone(),
                    Vault {
                        id: Uint128::new(i),
                        ..Vault::default()
                    },
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
    }
}
