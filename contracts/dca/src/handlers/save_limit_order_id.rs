use crate::error::ContractError;
use crate::helpers::message::get_attribute_in_event;
use crate::state::cache::VAULT_ID_CACHE;
use crate::state::triggers::save_trigger;
use crate::types::trigger::{Trigger, TriggerConfiguration};
use cosmwasm_std::{Decimal, DepsMut, Reply, Response, Uint128};

pub fn save_price_trigger(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let submit_order_response = reply.result.into_result().unwrap();

    let order_idx = get_attribute_in_event(&submit_order_response.events, "wasm", "order_idx")?
        .parse::<Uint128>()
        .expect("the order id of the submitted order");

    let target_price =
        get_attribute_in_event(&submit_order_response.events, "wasm", "target_price")?
            .parse::<Decimal>()
            .expect("the target price of the submitted order");

    let vault_id = VAULT_ID_CACHE.load(deps.storage)?;

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

    Ok(Response::new()
        .add_attribute("save_fin_limit_order_id", "true")
        .add_attribute("order_idx", order_idx))
}

#[cfg(test)]
mod save_limit_order_id_tests {
    use super::save_price_trigger;
    use crate::{
        state::{
            cache::VAULT_ID_CACHE,
            triggers::{get_trigger, save_trigger},
        },
        types::trigger::{Trigger, TriggerConfiguration},
    };
    use cosmwasm_std::{
        testing::mock_dependencies, Decimal, Event, Reply, SubMsgResponse, SubMsgResult, Uint128,
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
                events: vec![Event::new("wasm")
                    .add_attribute("order_idx", order_idx.to_string())
                    .add_attribute("target_price", Decimal::percent(200).to_string())],
                data: None,
            }),
        };

        save_price_trigger(deps.as_mut(), reply).unwrap();

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
}
