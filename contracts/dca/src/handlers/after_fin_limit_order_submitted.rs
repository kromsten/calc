use crate::error::ContractError;
use crate::state::old_cache::OLD_CACHE;
use crate::state::old_triggers::{get_old_trigger, save_old_trigger};
use base::helpers::message_helpers::get_attribute_in_event;
use base::triggers::trigger::{OldTrigger, OldTriggerConfiguration};
use cosmwasm_std::SubMsgResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Reply, Response, Uint128};

pub fn after_fin_limit_order_submitted(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response, ContractError> {
    match reply.result {
        SubMsgResult::Ok(_) => {
            let fin_submit_order_response = reply.result.into_result().unwrap();

            let order_idx =
                get_attribute_in_event(&fin_submit_order_response.events, "wasm", "order_idx")?
                    .parse::<Uint128>()
                    .expect("returned order_idx should be a valid Uint128");

            let cache = OLD_CACHE.load(deps.storage)?;

            let trigger = get_old_trigger(deps.storage, cache.vault_id)?
                .expect(format!("fin limit order trigger for vault {:?}", cache.vault_id).as_str());

            match trigger.configuration {
                OldTriggerConfiguration::FinLimitOrder { target_price, .. } => {
                    save_old_trigger(
                        deps.storage,
                        OldTrigger {
                            vault_id: cache.vault_id,
                            configuration: OldTriggerConfiguration::FinLimitOrder {
                                order_idx: Some(order_idx),
                                target_price,
                            },
                        },
                    )?;
                }
                _ => panic!("should be a fin limit order trigger"),
            }

            Ok(Response::new()
                .add_attribute("method", "fin_limit_order_submitted")
                .add_attribute("order_idx", order_idx))
        }
        SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!("failed to create vault with fin limit order trigger: {}", e),
        }),
    }
}
