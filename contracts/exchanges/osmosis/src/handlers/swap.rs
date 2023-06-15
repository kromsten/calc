use cosmwasm_std::{BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg};
use osmosis_std::types::osmosis::poolmanager::v1beta1::MsgSwapExactAmountIn;
use shared::coin::subtract;

use crate::{
    contract::AFTER_SWAP,
    helpers::routes::calculate_route,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        pairs::find_pair,
    },
    ContractError,
};

pub fn swap_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minimum_receive_amount: Coin,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidFunds {
            msg: "Must provide exactly one coin to swap".to_string(),
        });
    }

    if info.funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFunds {
            msg: "Must provide a non-zero amount to swap".to_string(),
        });
    }

    let pair = find_pair(
        deps.storage,
        [
            info.funds[0].denom.clone(),
            minimum_receive_amount.denom.clone(),
        ],
    )?;

    SWAP_CACHE.save(
        deps.storage,
        &SwapCache {
            sender: info.sender.clone(),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_denom_balance: deps.querier.query_balance(
                env.contract.address.clone(),
                minimum_receive_amount.denom.clone(),
            )?,
        },
    )?;

    let routes = calculate_route(&deps.querier, &pair, info.funds[0].denom.clone())?;

    Ok(Response::new()
        .add_attribute("swap", "true")
        .add_attribute("sender", info.sender)
        .add_attribute("swap_amount", info.funds[0].to_string())
        .add_attribute("minimum_receive_amount", minimum_receive_amount.to_string())
        .add_submessage(SubMsg {
            msg: MsgSwapExactAmountIn {
                sender: env.contract.address.to_string(),
                token_in: Some(info.funds[0].clone().into()),
                token_out_min_amount: minimum_receive_amount.amount.to_string(),
                routes,
            }
            .into(),
            id: AFTER_SWAP,
            reply_on: ReplyOn::Success,
            gas_limit: None,
        }))
}

pub fn return_swapped_funds(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    let updated_target_denom_balance = deps.querier.query_balance(
        env.contract.address,
        swap_cache.minimum_receive_amount.denom.clone(),
    )?;

    let return_amount = subtract(
        &updated_target_denom_balance,
        &swap_cache.target_denom_balance,
    )?;

    if return_amount.amount < swap_cache.minimum_receive_amount.amount {
        return Err(ContractError::FailedSwap {
            msg: format!(
                "{} is less than the minumum return amount of {}",
                return_amount, swap_cache.minimum_receive_amount
            ),
        });
    }

    Ok(Response::new()
        .add_attribute("return_amount", return_amount.to_string())
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: swap_cache.sender.to_string(),
            amount: vec![return_amount],
        })))
}

#[cfg(test)]
mod swap_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin, ReplyOn, StdError, SubMsg,
    };
    use osmosis_std::types::osmosis::poolmanager::v1beta1::{
        MsgSwapExactAmountIn, SwapAmountInRoute,
    };

    use crate::{
        contract::AFTER_SWAP,
        handlers::swap::swap_handler,
        state::{cache::SWAP_CACHE, pairs::save_pair},
        tests::{
            constants::{ADMIN, DENOM_UATOM, DENOM_UOSMO},
            mocks::calc_mock_dependencies,
        },
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
                kind: "osmosis::types::pair::Pair".to_string()
            })
        )
    }

    #[test]
    fn caches_details_correctly() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![Coin::new(0, DENOM_UATOM)],
        );

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
                .query_balance(env.contract.address, minimum_receive_amount.denom.clone())
                .unwrap()
        );
        assert_eq!(swap_cache.minimum_receive_amount, minimum_receive_amount);
    }

    #[test]
    fn sends_swap_message() {
        let mut deps = calc_mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom.clone())]);

        let minimum_receive_amount = Coin::new(3873213, pair.base_denom.clone());

        let response = swap_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            minimum_receive_amount.clone(),
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg {
                msg: MsgSwapExactAmountIn {
                    sender: mock_env().contract.address.to_string(),
                    token_in: Some(info.funds[0].clone().into()),
                    token_out_min_amount: minimum_receive_amount.amount.to_string(),
                    routes: vec![SwapAmountInRoute {
                        token_out_denom: pair.base_denom.clone(),
                        pool_id: pair.route[0]
                    }],
                }
                .into(),
                id: AFTER_SWAP,
                reply_on: ReplyOn::Success,
                gas_limit: None,
            }
        )
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
