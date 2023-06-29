use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

use crate::{
    error::ContractError,
    helpers::validation::assert_sender_is_admin,
    state::triggers::{get_trigger, save_trigger},
    types::trigger::{Trigger, TriggerConfiguration},
};

pub fn update_price_trigger(
    deps: DepsMut,
    info: MessageInfo,
    vault_id: Uint128,
    order_idx: Uint128,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    let trigger = get_trigger(deps.storage, vault_id)?;

    if let Some(Trigger {
        vault_id,
        configuration: TriggerConfiguration::Price { target_price, .. },
    }) = trigger
    {
        save_trigger(
            deps.storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Price {
                    order_idx,
                    target_price,
                },
            },
        )?;

        return Ok(Response::new()
            .add_attribute("update_price_trigger", "true")
            .add_attribute("vault_id", vault_id.to_string())
            .add_attribute("order_idx", order_idx.to_string()));
    }

    Err(ContractError::CustomError {
        val: "cannot update a non price trigger".to_string(),
    })
}

#[cfg(test)]
mod update_price_trigger_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Decimal, Timestamp, Uint128,
    };

    use crate::{
        handlers::update_price_trigger::update_price_trigger,
        state::triggers::get_trigger,
        tests::{
            helpers::{instantiate_contract, setup_vault},
            mocks::ADMIN,
        },
        types::{
            trigger::{Trigger, TriggerConfiguration},
            vault::Vault,
        },
    };

    #[test]
    fn with_non_price_trigger_fails() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        setup_vault(
            deps.as_mut(),
            env,
            Vault {
                id: Uint128::one(),
                trigger: Some(TriggerConfiguration::Time {
                    target_time: Timestamp::default(),
                }),
                ..Default::default()
            },
        );

        let err = update_price_trigger(deps.as_mut(), info, Uint128::one(), Uint128::new(12))
            .unwrap_err();

        assert_eq!(err.to_string(), "Error: cannot update a non price trigger");
    }

    #[test]
    fn updates_order_idx() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        setup_vault(
            deps.as_mut(),
            env,
            Vault {
                id: Uint128::one(),
                trigger: Some(TriggerConfiguration::Price {
                    order_idx: Uint128::one(),
                    target_price: Decimal::percent(213),
                }),
                ..Default::default()
            },
        );

        update_price_trigger(deps.as_mut(), info, Uint128::one(), Uint128::new(12)).unwrap();

        let trigger = get_trigger(deps.as_ref().storage, Uint128::one())
            .unwrap()
            .unwrap();

        assert_eq!(
            trigger,
            Trigger {
                vault_id: Uint128::one(),
                configuration: TriggerConfiguration::Price {
                    order_idx: Uint128::new(12),
                    target_price: Decimal::percent(213),
                }
            }
        );
    }
}
