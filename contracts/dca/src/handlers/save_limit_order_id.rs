use crate::error::ContractError;
use crate::helpers::message::get_attribute_in_event;
use crate::state::cache::VAULT_ID_CACHE;
use crate::state::triggers::{get_trigger, save_trigger};
use crate::types::trigger::{Trigger, TriggerConfiguration};
use cosmwasm_std::{DepsMut, Reply, Response, Uint128};

pub fn save_limit_order_id(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let fin_submit_order_response = reply.result.into_result().unwrap();

    let order_idx = get_attribute_in_event(&fin_submit_order_response.events, "wasm", "order_idx")?
        .parse::<Uint128>()
        .expect("returned order_idx should be a valid Uint128");

    let vault_id = VAULT_ID_CACHE.load(deps.storage)?;

    let trigger = get_trigger(deps.storage, vault_id)?;

    if trigger.is_none() {
        return Err(ContractError::CustomError {
            val: "Failed trying to save limit order id to a non-existent trigger".to_string(),
        });
    }

    if let TriggerConfiguration::Price { target_price, .. } = trigger.unwrap().configuration {
        save_trigger(
            deps.storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Price {
                    order_idx: Some(order_idx),
                    target_price,
                },
            },
        )?;
    } else {
        return Err(ContractError::CustomError {
            val: "Failed trying to save limit order id to a time trigger".to_string(),
        });
    }

    Ok(Response::new()
        .add_attribute("save_fin_limit_order_id", "true")
        .add_attribute("order_idx", order_idx))
}

#[cfg(test)]
mod save_limit_order_id_tests {
    use super::save_limit_order_id;
    use crate::{
        state::{
            cache::VAULT_ID_CACHE,
            triggers::{get_trigger, save_trigger},
        },
        types::trigger::{Trigger, TriggerConfiguration},
    };
    use cosmwasm_std::{
        testing::mock_dependencies, Decimal, Event, Reply, SubMsgResponse, SubMsgResult, Timestamp,
        Uint128,
    };

    #[test]
    fn should_save_limit_order_id() {
        let mut deps = mock_dependencies();

        let vault_id = Uint128::one();
        let order_idx = Uint128::new(67);

        VAULT_ID_CACHE
            .save(deps.as_mut().storage, &vault_id)
            .unwrap();

        save_trigger(
            deps.as_mut().storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Price {
                    target_price: Decimal::percent(200),
                    order_idx: None,
                },
            },
        )
        .unwrap();

        let reply = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm").add_attribute("order_idx", order_idx.to_string())],
                data: None,
            }),
        };

        save_limit_order_id(deps.as_mut(), reply).unwrap();

        let trigger = get_trigger(deps.as_ref().storage, vault_id).unwrap();

        assert_eq!(
            trigger,
            Some(Trigger {
                vault_id,
                configuration: TriggerConfiguration::Price {
                    target_price: Decimal::percent(200),
                    order_idx: Some(order_idx),
                },
            })
        );
    }

    #[test]
    fn for_time_trigger_should_fail() {
        let mut deps = mock_dependencies();

        let vault_id = Uint128::one();
        let order_idx = Uint128::new(67);

        VAULT_ID_CACHE
            .save(deps.as_mut().storage, &vault_id)
            .unwrap();

        save_trigger(
            deps.as_mut().storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Time {
                    target_time: Timestamp::default(),
                },
            },
        )
        .unwrap();

        let reply = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm").add_attribute("order_idx", order_idx.to_string())],
                data: None,
            }),
        };

        let err = save_limit_order_id(deps.as_mut(), reply).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: Failed trying to save limit order id to a time trigger"
        );
    }

    #[test]
    fn for_non_existent_trigger_should_fail() {
        let mut deps = mock_dependencies();

        let vault_id = Uint128::one();
        let order_idx = Uint128::new(67);

        VAULT_ID_CACHE
            .save(deps.as_mut().storage, &vault_id)
            .unwrap();

        let reply = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm").add_attribute("order_idx", order_idx.to_string())],
                data: None,
            }),
        };

        let err = save_limit_order_id(deps.as_mut(), reply).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: Failed trying to save limit order id to a non-existent trigger"
        );
    }
}
