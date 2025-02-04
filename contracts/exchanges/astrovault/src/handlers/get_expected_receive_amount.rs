use cosmwasm_std::{Coin, Deps, StdResult};
use crate::helpers::balance::coin_to_asset;
use crate::helpers::route::get_route_swap_simulate;
use crate::state::pairs::find_pair;


pub fn get_expected_receive_amount_handler(
    deps: Deps,
    swap_amount: Coin,
    target_denom: String,
) -> StdResult<Coin> {

    let pair = find_pair(
        deps.storage,
        [swap_amount.denom.clone(), target_denom.clone()],
    )?;

    let offer_asset = coin_to_asset(swap_amount);

    let amount = if pair.is_pool_pair() {
        pair.pool().swap_simulation(
            &deps.querier, 
            offer_asset,
        )?
    }  else {
        get_route_swap_simulate(
            deps,
            pair.route(),
            offer_asset,
        )?
    };

    Ok(Coin {
        denom: target_denom,
        amount,
    })

}

#[cfg(test)]
mod get_expected_receive_amount_handler_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, Coin, ContractResult, StdError, SystemResult,
        Uint128, to_json_binary,
    };

    use astrovault::standard_pool::query_msg::SimulationResponse;

    use crate::{
        handlers::get_expected_receive_amount::get_expected_receive_amount_handler,
        state::pairs::save_pair,
        tests::constants::{DENOM_AARCH, DENOM_UUSDC},
        types::pair::PopulatedPair,
    };

    #[test]
    fn for_missing_pair_fails() {

        let err = get_expected_receive_amount_handler(
            mock_dependencies().as_ref(),
            Coin {
                denom: DENOM_AARCH.to_string(),
                amount: Uint128::zero()
            },
            DENOM_UUSDC.to_string()
        ).unwrap_err();

        assert_eq!(err, StdError::generic_err("Pair not found"));
    }


    #[test]
    fn for_failed_simulation_fails() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Err("simulation failed".to_string()))
        });

        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                Coin {
                    denom: DENOM_AARCH.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UUSDC.to_string()
            )
            .unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: simulation failed".to_string()
            }
        )
    }

    #[test]
    fn for_successful_simulation_returns_expected_amount() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Ok(
                to_json_binary(&SimulationResponse {
                    return_amount: Uint128::from(83211293u128),
                    spread_amount: Uint128::default(),
                    commission_amount: Uint128::from(23312u128),
                    buybackburn_amount: Uint128::default(),
                })
                .unwrap(),
            ))
        });

        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                Coin {
                    denom: DENOM_AARCH.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UUSDC.to_string()
            )
            .unwrap(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: Uint128::from(83211293u128)
            }
        )
    }
}
