use cosmwasm_std::{Coin, Deps, DepsMut, Env, MessageInfo, Response};

use crate::ContractError;

pub fn swap_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _minimum_receive_amount: Coin,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn return_swapped_funds(_deps: Deps, _env: Env) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod swap_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info},
        Coin, StdError,
    };

    use crate::{
        handlers::swap::swap_handler,
        state::{cache::SWAP_CACHE, pairs::save_pair},
        tests::constants::{ADMIN, DENOM_UATOM, DENOM_UOSMO},
        types::pair::Pair,
        ContractError,
    };

    #[test]
    fn with_no_assets_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[]),
                Coin::new(12312, DENOM_UOSMO)
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: "Must provide exactly one coin to swap".to_string()
            }
        )
    }

    #[test]
    fn with_multiple_assets_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(
                    ADMIN,
                    &[Coin::new(12312, DENOM_UATOM), Coin::new(12312, DENOM_UOSMO)]
                ),
                Coin::new(12312, DENOM_UOSMO)
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: "Must provide exactly one coin to swap".to_string()
            }
        )
    }

    #[test]
    fn with_zero_swap_amount_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(0, DENOM_UOSMO)]),
                Coin::new(12312, DENOM_UOSMO)
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: "Must provide a non-zero amount to swap".to_string()
            }
        )
    }

    #[test]
    fn with_no_pair_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(12312, DENOM_UOSMO)]),
                Coin::new(12312, DENOM_UATOM)
            )
            .unwrap_err(),
            ContractError::Std(StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            })
        )
    }

    #[test]
    fn caches_details_correctly() {
        let mut deps = mock_dependencies_with_balance(&[Coin::new(0, DENOM_UATOM)]);

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom.clone())]);
        let minimum_receive_amount = Coin::new(3873213, pair.base_denom.clone());

        swap_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            minimum_receive_amount.clone(),
        )
        .unwrap();

        let swap_cache = SWAP_CACHE.load(deps.as_ref().storage).unwrap();

        assert_eq!(swap_cache.sender, ADMIN);
        assert_eq!(
            swap_cache.target_denom_balance,
            deps.as_ref()
                .querier
                .query_balance(
                    mock_env().contract.address,
                    minimum_receive_amount.denom.clone()
                )
                .unwrap()
        );
        assert_eq!(swap_cache.minimum_receive_amount, minimum_receive_amount);
    }

    #[test]
    fn sends_swap_message() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom.clone())]);

        let response = swap_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            Coin::new(3873213, pair.base_denom.clone()),
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        // assert_eq!(
        //     response.messages.first().unwrap(),
        //     &SubMsg::reply_on_success(
        //         PairContract(pair.address)
        //             .call(
        //                 ExecuteMsg::Swap {
        //                     offer_asset: None,
        //                     belief_price: None,
        //                     max_spread: None,
        //                     to: None,
        //                     callback: None
        //                 },
        //                 info.funds
        //             )
        //             .unwrap(),
        //         AFTER_SWAP
        //     )
        // )
    }
}

#[cfg(test)]
mod return_swapped_funds_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, BankMsg, Coin,
    };
    use shared::coin::{add, empty_of};

    use crate::{
        handlers::swap::return_swapped_funds,
        state::cache::{SwapCache, SWAP_CACHE},
        tests::constants::DENOM_UOSMO,
        ContractError,
    };

    #[test]
    fn with_return_amount_smaller_than_minimum_receive_amount_fails() {
        let mut deps = mock_dependencies();

        let minimum_receive_amount = Coin::new(123, DENOM_UOSMO);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_denom_balance: Coin::new(122, DENOM_UOSMO),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        assert_eq!(
            return_swapped_funds(deps.as_ref(), mock_env()).unwrap_err(),
            ContractError::FailedSwap {
                msg: format!(
                    "{} is less than the minumum return amount of {}",
                    empty_of(minimum_receive_amount.clone()),
                    minimum_receive_amount
                )
            }
        )
    }

    #[test]
    fn sends_funds_back_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let minimum_receive_amount = Coin::new(123, DENOM_UOSMO);
        let target_denom_balance = Coin::new(122, DENOM_UOSMO);
        let return_amount = Coin::new(153, DENOM_UOSMO);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount,
            target_denom_balance: target_denom_balance.clone(),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![add(target_denom_balance, return_amount.clone()).unwrap()],
        );

        let response = return_swapped_funds(deps.as_ref(), env).unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &cosmwasm_std::SubMsg::new(BankMsg::Send {
                to_address: swap_cache.sender.to_string(),
                amount: vec![return_amount],
            })
        )
    }
}
