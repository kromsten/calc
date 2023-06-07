use cosmwasm_std::{Coin, Deps, StdResult};
use kujira_fin::{QueryMsg, SimulationResponse};
use kujira_std::{Asset, AssetInfo, Denom};

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

    let simulation = deps.querier.query_wasm_smart::<SimulationResponse>(
        pair.address,
        &QueryMsg::Simulation {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: Denom::from(swap_amount.denom.clone()),
                },
                amount: swap_amount.amount,
            },
        },
    )?;

    Ok(Coin {
        denom: target_denom,
        amount: simulation.return_amount.try_into()?,
    })
}

#[cfg(test)]
mod get_expected_receive_amount_handler_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, to_binary, Coin, ContractResult, StdError, SystemResult,
        Uint128, Uint256,
    };
    use kujira_fin::SimulationResponse;

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

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&SimulationResponse {
                    return_amount: Uint256::from(83211293u128),
                    spread_amount: Uint256::from(13312u128),
                    commission_amount: Uint256::from(23312u128),
                })
                .unwrap(),
            ))
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
            .unwrap(),
            Coin {
                denom: DENOM_UUSK.to_string(),
                amount: Uint128::from(83211293u128)
            }
        )
    }
}
