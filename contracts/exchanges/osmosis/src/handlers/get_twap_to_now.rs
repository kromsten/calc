use cosmwasm_std::{Decimal, Decimal256, Deps, Env, StdResult};
use osmosis_std::{shim::Timestamp, types::osmosis::twap::v1beta1::TwapQuerier};

use crate::{
    helpers::routes::get_token_out_denom, state::pairs::find_pair,
    types::position_type::PositionType,
};

pub fn get_twap_to_now_handler(
    deps: Deps,
    env: Env,
    mut swap_denom: String,
    target_denom: String,
    period: u64,
) -> StdResult<Decimal256> {
    let pair = find_pair(deps.storage, [swap_denom.clone(), target_denom])?;

    let route = match pair.position_type(swap_denom.clone()) {
        PositionType::Enter => pair.route,
        PositionType::Exit => pair.route.into_iter().rev().collect(),
    };

    let mut price = Decimal::one();

    for pool_id in route.into_iter() {
        let target_denom = get_token_out_denom(&deps.querier, swap_denom.clone(), pool_id)?;

        let pool_price = TwapQuerier::new(&deps.querier)
            .arithmetic_twap_to_now(
                pool_id,
                target_denom.clone(),
                swap_denom.clone(),
                Some(Timestamp {
                    seconds: (env.block.time.seconds() - period) as i64,
                    nanos: 0,
                }),
            )
            .unwrap()
            .arithmetic_twap
            .parse::<Decimal>()?;

        price = pool_price * price;

        swap_denom = target_denom;
    }

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
                0
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

        let price =
            get_twap_to_now_handler(deps.as_ref(), env, pair.quote_denom, pair.base_denom, 60)
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

        let price =
            get_twap_to_now_handler(deps.as_ref(), env, pair.quote_denom, pair.base_denom, 60)
                .unwrap();

        assert_eq!(price, Decimal256::percent(20) * Decimal256::percent(120));
    }
}
