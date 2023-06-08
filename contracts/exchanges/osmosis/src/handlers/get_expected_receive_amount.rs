use cosmwasm_std::{Coin, Deps, StdResult};

pub fn get_expected_receive_amount_handler(
    _deps: Deps,
    _swap_amount: Coin,
    _target_denom: String,
) -> StdResult<Coin> {
    unimplemented!()
}

#[cfg(test)]
mod get_expected_receive_amount_handler_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, Coin, ContractResult, StdError, SystemResult, Uint128,
    };

    use crate::{
        handlers::get_expected_receive_amount::get_expected_receive_amount_handler,
        state::pairs::save_pair,
        tests::constants::{DENOM_UKUJI, DENOM_UUSK},
        types::pair::Pair,
    };

    #[test]
    fn for_missing_pair_fails() {
        assert_eq!(
            get_expected_receive_amount_handler(
                mock_dependencies().as_ref(),
                Coin {
                    denom: DENOM_UKUJI.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UUSK.to_string()
            )
            .unwrap_err(),
            StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            }
        )
    }

    #[test]
    fn for_failed_simulation_fails() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Err("simulation failed".to_string()))
        });

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                Coin {
                    denom: DENOM_UKUJI.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UUSK.to_string()
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

        // deps.querier.update_wasm(|_| {
        //     SystemResult::Ok(ContractResult::Ok(
        //         to_binary(&SimulationResponse {
        //             return_amount: Uint256::from(83211293u128),
        //             spread_amount: Uint256::from(13312u128),
        //             commission_amount: Uint256::from(23312u128),
        //         })
        //         .unwrap(),
        //     ))
        // });

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_expected_receive_amount_handler(
                deps.as_ref(),
                Coin {
                    denom: DENOM_UKUJI.to_string(),
                    amount: Uint128::zero()
                },
                DENOM_UUSK.to_string()
            )
            .unwrap(),
            Coin {
                denom: DENOM_UUSK.to_string(),
                amount: Uint128::from(83211293u128)
            }
        )
    }
}
