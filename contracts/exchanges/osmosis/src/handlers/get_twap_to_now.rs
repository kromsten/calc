use cosmwasm_std::{Decimal, Decimal256, Deps, Env, QuerierWrapper, StdError, StdResult};
use osmosis_std::{
    shim::Timestamp,
    types::osmosis::{
        gamm::v1beta1::{GammQuerier, Pool},
        twap::v1beta1::TwapQuerier,
    },
};
use prost::DecodeError;

use crate::{state::pairs::find_pair, types::position_type::PositionType};

pub fn get_twap_to_now_handler(
    deps: Deps,
    env: Env,
    mut swap_denom: String,
    target_denom: String,
    period: u64,
) -> StdResult<Decimal256> {
    let pair = find_pair(deps.storage, [swap_denom.clone(), target_denom])?;

    let route = match pair.position_type(swap_denom.clone()) {
        PositionType::Enter => pair.route.clone(),
        PositionType::Exit => pair.route.clone().into_iter().rev().collect(),
    };

    let mut price = Decimal::one();

    for pool_id in route.into_iter() {
        let target_denom = get_token_out_denom(&deps.querier, swap_denom.clone(), pool_id)?;

        let pool = get_pool(&deps.querier, pool_id)?;

        let swap_fee = pool
            .pool_params
            .unwrap()
            .swap_fee
            .parse::<Decimal>()
            .unwrap();

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
            .parse::<Decimal>()?
            * (Decimal::one() + swap_fee);

        price = pool_price * price;

        swap_denom = target_denom;
    }

    Ok(price.into())
}

fn get_token_out_denom(
    querier: &QuerierWrapper,
    token_in_denom: String,
    pool_id: u64,
) -> StdResult<String> {
    let pool = get_pool(querier, pool_id)?;

    if pool.pool_assets.len() != 2 {
        return Err(StdError::generic_err(format!(
            "pool id {} is not a 2 asset pool",
            pool_id
        )));
    }

    if pool
        .pool_assets
        .iter()
        .all(|asset| asset.token.clone().unwrap().denom != token_in_denom)
    {
        return Err(StdError::generic_err(format!(
            "denom {} not found in pool id {}",
            token_in_denom, pool_id
        )));
    }

    let token_out_denom = pool
        .pool_assets
        .iter()
        .find(|asset| asset.token.clone().unwrap().denom != token_in_denom)
        .map(|asset| asset.token.clone().unwrap().denom)
        .ok_or_else(|| StdError::generic_err("no token out denom found"));

    token_out_denom
}

pub fn get_pool(querier: &QuerierWrapper, pool_id: u64) -> Result<Pool, StdError> {
    GammQuerier::new(querier).pool(pool_id)?.pool.map_or(
        Err(StdError::generic_err("pool not found")),
        |pool| {
            pool.try_into()
                .map_err(|e: DecodeError| StdError::ParseErr {
                    target_type: Pool::TYPE_URL.to_string(),
                    msg: e.to_string(),
                })
        },
    )
}

#[cfg(test)]
mod get_twap_to_now_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Decimal256, StdError,
    };

    use crate::{
        handlers::get_twap_to_now::get_twap_to_now_handler,
        state::pairs::save_pair,
        tests::constants::{DENOM_UATOM, DENOM_UOSMO},
        types::pair::Pair,
    };

    #[test]
    fn with_period_larger_than_zero_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                mock_env(),
                DENOM_UOSMO.to_string(),
                DENOM_UATOM.to_string(),
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
                mock_env(),
                DENOM_UOSMO.to_string(),
                DENOM_UATOM.to_string(),
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

        // deps.querier.update_wasm(|_| {
        //     SystemResult::Ok(ContractResult::Ok(
        //         to_binary(&BookResponse {
        //             base: vec![],
        //             quote: vec![],
        //         })
        //         .unwrap(),
        //     ))
        // });

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                mock_env(),
                DENOM_UOSMO.to_string(),
                DENOM_UATOM.to_string(),
                0
            )
            .unwrap_err(),
            StdError::generic_err(format!(
                "No orders found for {} at fin pair {:?}",
                DENOM_UOSMO, pair
            ))
        )
    }

    #[test]
    fn for_fin_buy_returns_quote_price() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        // deps.querier.update_wasm(move |_| {
        //     SystemResult::Ok(ContractResult::Ok(
        //         to_binary(&BookResponse {
        //             base: vec![PoolResponse {
        //                 quote_price: Decimal256::percent(50),
        //                 offer_denom: Denom::Native(pair.base_denom.to_string()),
        //                 total_offer_amount: Uint256::from_u128(372u128),
        //             }],
        //             quote: vec![PoolResponse {
        //                 quote_price: Decimal256::percent(30),
        //                 offer_denom: Denom::Native(pair.quote_denom.to_string()),
        //                 total_offer_amount: Uint256::from_u128(372u128),
        //             }],
        //         })
        //         .unwrap(),
        //     ))
        // });

        let pair = Pair::default();

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                mock_env(),
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

        // deps.querier.update_wasm(move |_| {
        //     SystemResult::Ok(ContractResult::Ok(
        //         to_binary(&BookResponse {
        //             base: vec![PoolResponse {
        //                 quote_price: Decimal256::percent(50),
        //                 offer_denom: Denom::Native(pair.base_denom.to_string()),
        //                 total_offer_amount: Uint256::from_u128(372u128),
        //             }],
        //             quote: vec![PoolResponse {
        //                 quote_price: Decimal256::percent(30),
        //                 offer_denom: Denom::Native(pair.quote_denom.to_string()),
        //                 total_offer_amount: Uint256::from_u128(372u128),
        //             }],
        //         })
        //         .unwrap(),
        //     ))
        // });

        let pair = Pair::default();

        assert_eq!(
            get_twap_to_now_handler(
                deps.as_ref(),
                mock_env(),
                pair.base_denom.to_string(),
                pair.quote_denom.to_string(),
                0
            )
            .unwrap(),
            Decimal256::one() / Decimal256::percent(30)
        )
    }
}
