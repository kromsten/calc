use cosmwasm_std::{from_json, Binary, Decimal, Decimal256, Deps, Env, StdError, StdResult};
use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountInRoute;

use crate::{
    helpers::{price::get_arithmetic_twap_to_now, routes::get_token_out_denom},
    state::pairs::find_pair,
    types::position_type::PositionType,
};

pub fn get_twap_to_now_handler(
    deps: Deps,
    env: Env,
    mut swap_denom: String,
    target_denom: String,
    period: u64,
    injected_route: Option<Binary>,
) -> StdResult<Decimal256> {
    let route = injected_route.map_or_else(
        || {
            let pair = find_pair(deps.storage, [swap_denom.clone(), target_denom.clone()])?;

            Ok(match pair.position_type(swap_denom.clone()) {
                PositionType::Enter => pair.route,
                PositionType::Exit => pair.route.into_iter().rev().collect(),
            })
        },
        |r| {
            from_json::<Vec<SwapAmountInRoute>>(r.as_slice()).map_or_else(
                |e| Err(StdError::generic_err(e.to_string())),
                |r| {
                    Ok(r.into_iter()
                        .map(|r: SwapAmountInRoute| r.pool_id)
                        .collect::<Vec<u64>>())
                },
            )
        },
    )?;

    let mut price = Decimal::one();

    for adjacent_pools in route.windows(2).into_iter() {
        let token_out_denom = get_token_out_denom(
            &deps.querier,
            swap_denom.clone(),
            adjacent_pools[0],
            adjacent_pools[1],
        )?;

        let pool_price = get_arithmetic_twap_to_now(
            &deps.querier,
            env.clone(),
            adjacent_pools[0],
            swap_denom,
            token_out_denom.clone(),
            period,
        )?;

        price = pool_price * price;

        swap_denom = token_out_denom;
    }

    let final_pool_price = get_arithmetic_twap_to_now(
        &deps.querier,
        env,
        *route.last().unwrap(),
        swap_denom,
        target_denom,
        period,
    )?;

    price = final_pool_price * price;

    Ok(price.into())
}

#[cfg(test)]
mod get_twap_to_now_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        to_json_binary, Decimal256, StdError,
    };
    use osmosis_std::types::osmosis::twap::v1beta1::{
        ArithmeticTwapRequest, ArithmeticTwapResponse,
    };
    use prost::Message;

    use crate::{
        handlers::get_twap_to_now::get_twap_to_now_handler,
        state::pairs::save_pair,
        tests::{
            constants::{DENOM_UATOM, DENOM_UOSMO},
            mocks::calc_mock_dependencies,
        },
        types::pair::Pair,
    };

    #[test]
    fn with_no_pair_for_denoms_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                mock_env(),
                DENOM_UOSMO.to_string(),
                DENOM_UATOM.to_string(),
                0,
                None
            )
            .unwrap_err(),
            StdError::NotFound {
                kind: "osmosis::types::pair::Pair".to_string()
            }
        )
    }

    #[test]
    fn query_belief_price_with_single_pool_id_should_succeed() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();

        deps.querier.update_stargate(|path, data| {
            if path == "/osmosis.twap.v1beta1.Query/ArithmeticTwapToNow" {
                let price = match ArithmeticTwapRequest::decode(data.as_slice())
                    .unwrap()
                    .pool_id
                {
                    3 => "0.8",
                    _ => "1.0",
                };

                return to_json_binary(&ArithmeticTwapResponse {
                    arithmetic_twap: price.to_string(),
                });
            }
            Err(StdError::generic_err("invoke fallback"))
        });

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let price = get_twap_to_now_handler(
            deps.as_ref(),
            env,
            pair.quote_denom,
            pair.base_denom,
            60,
            None,
        )
        .unwrap();

        assert_eq!(price, Decimal256::percent(80));
    }

    #[test]
    fn query_belief_price_with_multiple_pool_ids_id_should_succeed() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();

        deps.querier.update_stargate(|path, data| {
            if path == "/osmosis.twap.v1beta1.Query/ArithmeticTwapToNow" {
                let price = match ArithmeticTwapRequest::decode(data.as_slice())
                    .unwrap()
                    .pool_id
                {
                    1 => "0.2",
                    4 => "1.2",
                    _ => "1.0",
                };

                return to_json_binary(&ArithmeticTwapResponse {
                    arithmetic_twap: price.to_string(),
                });
            }
            Err(StdError::generic_err("invoke fallback"))
        });

        let pair = Pair {
            route: vec![4, 1],
            ..Pair::default()
        };

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let price = get_twap_to_now_handler(
            deps.as_ref(),
            env,
            pair.quote_denom,
            pair.base_denom,
            60,
            None,
        )
        .unwrap();

        assert_eq!(price, Decimal256::percent(20) * Decimal256::percent(120));
    }
}
