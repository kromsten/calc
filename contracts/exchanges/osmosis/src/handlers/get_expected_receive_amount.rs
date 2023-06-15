use cosmwasm_std::{Coin, Deps, Env, StdError, StdResult, Uint128};
use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;

use crate::{helpers::routes::calculate_route, state::pairs::find_pair};

pub fn get_expected_receive_amount_handler(
    deps: Deps,
    env: Env,
    swap_amount: Coin,
    target_denom: String,
) -> StdResult<Coin> {
    let pair = find_pair(
        deps.storage,
        [swap_amount.denom.clone(), target_denom.clone()],
    )?;

    let routes = calculate_route(&deps.querier, &pair, swap_amount.denom.clone())?;

    let token_out_amount = PoolmanagerQuerier::new(&deps.querier)
        .estimate_swap_exact_amount_in(
            env.contract.address.to_string(),
            0,
            swap_amount.to_string(),
            routes.clone(),
        )
        .map_err(|_| {
            StdError::generic_err(format!(
                "amount of {} received for swapping {} via {:#?}",
                routes.last().unwrap().token_out_denom,
                swap_amount,
                routes
            ))
        })?
        .token_out_amount
        .parse::<Uint128>()?;

    Ok(Coin::new(token_out_amount.into(), target_denom))
}

#[cfg(test)]
mod get_expected_receive_amount_handler_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Coin, StdError, Uint128,
    };

    use crate::{
        handlers::get_expected_receive_amount::get_expected_receive_amount_handler,
        state::pairs::save_pair,
        tests::{
            constants::{DENOM_UATOM, DENOM_UOSMO},
            mocks::calc_mock_dependencies,
        },
        types::pair::Pair,
    };

    #[test]
    fn for_missing_pair_fails() {
        assert_eq!(
            get_expected_receive_amount_handler(
                mock_dependencies().as_ref(),
                mock_env(),
                Coin {
                    denom: DENOM_UOSMO.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UATOM.to_string()
            )
            .unwrap_err(),
            StdError::NotFound {
                kind: "osmosis::types::pair::Pair".to_string()
            }
        )
    }

    #[test]
    fn for_successful_simulation_returns_expected_amount() {
        let mut deps = calc_mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                mock_env(),
                Coin {
                    denom: pair.base_denom.to_string(),
                    amount: Uint128::zero()
                },
                pair.quote_denom.to_string()
            )
            .unwrap(),
            Coin {
                amount: Uint128::new(1231232),
                denom: pair.quote_denom,
            }
        )
    }
}
