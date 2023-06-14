use cosmwasm_std::{Deps, StdResult, Uint128};
use exchange::order::Order;

pub fn get_order_handler(
    _deps: Deps,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> StdResult<Order> {
    unimplemented!()
}

#[cfg(test)]
mod get_order_handler_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, Coin, ContractResult, StdError, SystemResult, Uint128, Uint256,
    };
    use exchange::order::Order;

    use crate::{
        handlers::get_order::get_order_handler,
        state::pairs::save_pair,
        tests::constants::{DENOM_UATOM, DENOM_UOSMO, ONE},
        types::pair::Pair,
    };

    #[test]
    fn for_missing_pair_fails() {
        assert_eq!(
            get_order_handler(
                mock_dependencies().as_ref(),
                Uint128::zero(),
                [DENOM_UOSMO.to_string(), DENOM_UATOM.to_string()]
            )
            .unwrap_err(),
            StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            }
        )
    }

    #[test]
    fn for_missing_order_fails() {
        let mut deps = mock_dependencies();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Err(
                "No orders with the specified information exist".to_string(),
            ))
        });

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        assert_eq!(
            get_order_handler(deps.as_ref(), Uint128::zero(), pair.denoms()).unwrap_err(),
            StdError::generic_err(
                "Querier contract error: No orders with the specified information exist"
                    .to_string()
            )
        )
    }

    #[test]
    fn for_valid_order_returns_order() {
        let mut deps = mock_dependencies();

        // deps.querier.update_wasm(|_| {
        //     SystemResult::Ok(ContractResult::Ok(
        //         to_binary(&OrderResponse {
        //             original_offer_amount: Uint256::from_u128(13123213u128),
        //             offer_amount: Uint256::from_u128(2u128),
        //             filled_amount: Uint256::from_u128(3223423u128),
        //             idx: ONE,
        //             owner: Addr::unchecked(ADMIN),
        //             quote_price: Decimal256::percent(213921),
        //             offer_denom: Denom::Native(DENOM_UKUJI.to_string()),
        //             created_at: Timestamp::default(),
        //         })
        //         .unwrap(),
        //     ))
        // });

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order = get_order_handler(deps.as_ref(), ONE, pair.denoms()).unwrap();

        assert_eq!(
            order,
            Order {
                order_idx: ONE,
                remaining_offer_amount: Coin {
                    amount: Uint256::from_u128(2u128).try_into().unwrap(),
                    denom: DENOM_UOSMO.to_string(),
                },
            }
        );
    }
}
