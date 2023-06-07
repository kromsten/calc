use cosmwasm_std::{Decimal256, Deps, StdError, StdResult};
use kujira_fin::{BookResponse, QueryMsg};

use crate::{state::pairs::find_pair, types::position_type::PositionType};

pub fn get_twap_to_now_handler(
    deps: Deps,
    swap_denom: String,
    target_denom: String,
    period: u64,
) -> StdResult<Decimal256> {
    if period != 0 {
        return Err(StdError::generic_err(format!(
            "Cannot get twap for period of {} seconds, only 0 is supported",
            period
        )));
    }

    let pair = find_pair(deps.storage, [swap_denom.clone(), target_denom])?;

    let position_type = match swap_denom == pair.quote_denom {
        true => PositionType::Enter,
        false => PositionType::Exit,
    };

    let book_response = deps.querier.query_wasm_smart::<BookResponse>(
        pair.address.clone(),
        &QueryMsg::Book {
            limit: Some(1),
            offset: None,
        },
    )?;

    let book = match position_type {
        PositionType::Enter => book_response.base,
        PositionType::Exit => book_response.quote,
    };

    if book.is_empty() {
        return Err(StdError::generic_err(format!(
            "No orders found for {} at fin pair {}",
            swap_denom, pair.address
        )));
    }

    let quote_price = book[0].quote_price;

    Ok(match position_type {
        PositionType::Enter => quote_price,
        PositionType::Exit => Decimal256::one()
            .checked_div(quote_price)
            .expect("should return a valid inverted price for fin sell"),
    })
}

#[cfg(test)]
mod get_twap_to_now_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, to_binary, ContractResult, Decimal256, StdError, SystemResult,
        Uint256,
    };
    use cw20::Denom;
    use kujira_fin::{BookResponse, PoolResponse};

    use crate::{
        handlers::get_twap_to_now::get_twap_to_now_handler,
        state::pairs::save_pair,
        tests::constants::{DENOM_UKUJI, DENOM_UUSK},
        types::pair::Pair,
    };

    #[test]
    fn with_period_larger_than_zero_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                DENOM_UKUJI.to_string(),
                DENOM_UUSK.to_string(),
                10
            )
            .unwrap_err(),
            StdError::generic_err("Cannot get twap for period of 10 seconds, only 0 is supported")
        )
    }

    #[test]
    fn with_no_pair_for_denoms_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                DENOM_UKUJI.to_string(),
                DENOM_UUSK.to_string(),
                0
            )
            .unwrap_err(),
            StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            }
        )
    }

    #[test]
    fn with_no_orders_for_denom_fails() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&BookResponse {
                    base: vec![],
                    quote: vec![],
                })
                .unwrap(),
            ))
        });

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                DENOM_UKUJI.to_string(),
                DENOM_UUSK.to_string(),
                0
            )
            .unwrap_err(),
            StdError::generic_err(format!(
                "No orders found for {} at fin pair {}",
                DENOM_UKUJI, pair.address
            ))
        )
    }

    #[test]
    fn for_fin_buy_returns_quote_price() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&BookResponse {
                    base: vec![PoolResponse {
                        quote_price: Decimal256::percent(50),
                        offer_denom: Denom::Native(pair.base_denom.to_string()),
                        total_offer_amount: Uint256::from_u128(372u128),
                    }],
                    quote: vec![PoolResponse {
                        quote_price: Decimal256::percent(30),
                        offer_denom: Denom::Native(pair.quote_denom.to_string()),
                        total_offer_amount: Uint256::from_u128(372u128),
                    }],
                })
                .unwrap(),
            ))
        });

        let pair = Pair::default();

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                pair.quote_denom.to_string(),
                pair.base_denom.to_string(),
                0
            )
            .unwrap(),
            Decimal256::percent(50)
        )
    }

    #[test]
    fn for_fin_sell_returns_inverted_quote_price() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&BookResponse {
                    base: vec![PoolResponse {
                        quote_price: Decimal256::percent(50),
                        offer_denom: Denom::Native(pair.base_denom.to_string()),
                        total_offer_amount: Uint256::from_u128(372u128),
                    }],
                    quote: vec![PoolResponse {
                        quote_price: Decimal256::percent(30),
                        offer_denom: Denom::Native(pair.quote_denom.to_string()),
                        total_offer_amount: Uint256::from_u128(372u128),
                    }],
                })
                .unwrap(),
            ))
        });

        let pair = Pair::default();

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                pair.base_denom.to_string(),
                pair.quote_denom.to_string(),
                0
            )
            .unwrap(),
            Decimal256::one() / Decimal256::percent(30)
        )
    }
}
