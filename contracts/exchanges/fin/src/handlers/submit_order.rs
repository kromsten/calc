use cosmwasm_std::{Decimal256, Deps, MessageInfo, Reply, Response, SubMsg, Uint128};
use kujira_fin::ExecuteMsg;

use crate::{
    contract::AFTER_SUBMIT_ORDER,
    helpers::{message::get_attribute_in_event, price::get_fin_price},
    state::pairs::find_pair,
    types::pair_contract::PairContract,
    ContractError,
};

pub fn submit_order_handler(
    deps: Deps,
    info: MessageInfo,
    target_price: Decimal256,
    target_denom: String,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidFunds {
            msg: String::from("must send exactly one asset"),
        });
    }

    if info.funds[0].denom.clone() == target_denom {
        return Err(ContractError::InvalidFunds {
            msg: String::from("swap denom and target denom must be different"),
        });
    }

    let pair = find_pair(deps.storage, [info.funds[0].denom.clone(), target_denom])?;

    let price = get_fin_price(&deps.querier, target_price, &info.funds[0].denom, &pair)?;

    Ok(Response::new()
        .add_attribute("submit_order", "true")
        .add_attribute("target_price", price.to_string())
        .add_submessage(SubMsg::reply_on_success(
            PairContract(pair.address).call(
                ExecuteMsg::SubmitOrder {
                    price,
                    callback: None,
                },
                info.funds,
            )?,
            AFTER_SUBMIT_ORDER,
        )))
}

pub fn return_order_idx(reply: Reply) -> Result<Response, ContractError> {
    let order_idx = get_attribute_in_event(
        &reply.result.into_result().unwrap().events,
        "wasm",
        "order_idx",
    )?
    .parse::<Uint128>()
    .unwrap();

    Ok(Response::new().add_attribute("order_idx", order_idx))
}

#[cfg(test)]
mod submit_order_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_info},
        to_binary, Addr, Coin, ContractResult, Decimal256, StdError, SubMsg, SystemResult, WasmMsg,
    };
    use cw20::Denom;
    use kujira_fin::{ConfigResponse, ExecuteMsg};
    use kujira_std::{Precise, Precision};

    use crate::{
        contract::AFTER_SUBMIT_ORDER,
        state::pairs::save_pair,
        tests::constants::{ADMIN, DENOM_UKUJI, DENOM_UUSK},
        types::pair::Pair,
        ContractError,
    };

    use super::*;

    #[test]
    fn with_no_assets_fails() {
        assert_eq!(
            submit_order_handler(
                mock_dependencies().as_ref(),
                mock_info(ADMIN, &[]),
                Decimal256::one(),
                DENOM_UKUJI.to_string(),
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: String::from("must send exactly one asset")
            }
        );
    }

    #[test]
    fn with_more_than_one_asset_fails() {
        assert_eq!(
            submit_order_handler(
                mock_dependencies().as_ref(),
                mock_info(
                    ADMIN,
                    &[Coin::new(43282, DENOM_UKUJI), Coin::new(234782, DENOM_UUSK)]
                ),
                Decimal256::one(),
                DENOM_UKUJI.to_string(),
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: String::from("must send exactly one asset")
            }
        );
    }

    #[test]
    fn with_the_same_swap_and_target_denom_fails() {
        assert_eq!(
            submit_order_handler(
                mock_dependencies().as_ref(),
                mock_info(ADMIN, &[Coin::new(43282, DENOM_UKUJI)]),
                Decimal256::one(),
                DENOM_UKUJI.to_string(),
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: String::from("swap denom and target denom must be different")
            }
        );
    }

    #[test]
    fn with_no_matching_pair_fails() {
        assert_eq!(
            submit_order_handler(
                mock_dependencies().as_ref(),
                mock_info(ADMIN, &[Coin::new(43282, DENOM_UUSK)]),
                Decimal256::one(),
                DENOM_UKUJI.to_string(),
            )
            .unwrap_err(),
            ContractError::Std(StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            })
        );
    }

    #[test]
    fn sends_submit_order_message() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(3),
                    decimal_delta: 0,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::percent(24312);

        let info = mock_info(ADMIN, &[Coin::new(123123, pair.quote_denom)]);

        let response = submit_order_handler(
            deps.as_ref(),
            info.clone(),
            target_price,
            pair.base_denom.to_string(),
        )
        .unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: pair.address.to_string(),
                    msg: to_binary(&ExecuteMsg::SubmitOrder {
                        price: target_price,
                        callback: None
                    })
                    .unwrap(),
                    funds: info.funds
                },
                AFTER_SUBMIT_ORDER
            )
        );
    }

    #[test]
    fn inverts_price_for_fin_sell() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(3),
                    decimal_delta: 0,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::percent(24312);

        let info = mock_info(ADMIN, &[Coin::new(123123, pair.base_denom)]);

        let response = submit_order_handler(
            deps.as_ref(),
            info.clone(),
            target_price,
            pair.quote_denom.to_string(),
        )
        .unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: pair.address.to_string(),
                    msg: to_binary(&ExecuteMsg::SubmitOrder {
                        price: (Decimal256::one() / target_price)
                            .round(&Precision::DecimalPlaces(3)),
                        callback: None
                    })
                    .unwrap(),
                    funds: info.funds
                },
                AFTER_SUBMIT_ORDER
            )
        );
    }
}
