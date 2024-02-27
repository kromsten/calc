use cosmwasm_std::{BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, SubMsg};
use kujira_fin::ExecuteMsg;
use shared::{balance::query_balance, coin::subtract};

use crate::{
    contract::AFTER_SWAP,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        pairs::find_pair,
    },
    types::pair_contract::PairContract,
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
            target_denom_balance: query_balance(
                deps.api,
                &deps.querier,
                &minimum_receive_amount.denom,
                &env.contract.address,
            )?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("swap", "true")
        .add_attribute("sender", info.sender)
        .add_attribute("swap_amount", info.funds[0].to_string())
        .add_attribute("minimum_receive_amount", minimum_receive_amount.to_string())
        .add_submessage(SubMsg::reply_on_success(
            PairContract(pair.address).call(
                ExecuteMsg::Swap {
                    offer_asset: None,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                    callback: None,
                },
                info.funds,
            )?,
            AFTER_SWAP,
        )))
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
                "{} is less than the minimum return amount of {}",
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
        testing::{mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info},
        Coin, StdError, SubMsg,
    };
    use kujira_fin::ExecuteMsg;

    use crate::{
        contract::AFTER_SWAP,
        handlers::swap::swap_handler,
        state::{cache::SWAP_CACHE, pairs::save_pair},
        tests::constants::{ADMIN, DENOM_UKUJI, DENOM_UUSK},
        types::{pair::Pair, pair_contract::PairContract},
        ContractError,
    };

    #[test]
    fn with_no_assets_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[]),
                Coin::new(12312, DENOM_UKUJI)
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
                    &[Coin::new(12312, DENOM_UUSK), Coin::new(12312, DENOM_UKUJI)]
                ),
                Coin::new(12312, DENOM_UKUJI)
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
                mock_info(ADMIN, &[Coin::new(0, DENOM_UKUJI)]),
                Coin::new(12312, DENOM_UKUJI)
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
                mock_info(ADMIN, &[Coin::new(12312, DENOM_UKUJI)]),
                Coin::new(12312, DENOM_UUSK)
            )
            .unwrap_err(),
            ContractError::Std(StdError::NotFound {
                kind: "fin::types::pair::Pair".to_string()
            })
        )
    }

    #[test]
    fn caches_details_correctly() {
        let mut deps = mock_dependencies_with_balance(&[Coin::new(0, DENOM_UUSK)]);

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom.clone())]);
        let minimum_receive_amount = Coin::new(3873213, pair.base_denom);

        swap_handler(
            deps.as_mut(),
            mock_env(),
            info,
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

        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::reply_on_success(
                PairContract(pair.address)
                    .call(
                        ExecuteMsg::Swap {
                            offer_asset: None,
                            belief_price: None,
                            max_spread: None,
                            to: None,
                            callback: None
                        },
                        info.funds
                    )
                    .unwrap(),
                AFTER_SWAP
            )
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
        tests::constants::DENOM_UKUJI,
        ContractError,
    };

    #[test]
    fn with_return_amount_smaller_than_minimum_receive_amount_fails() {
        let mut deps = mock_dependencies();

        let minimum_receive_amount = Coin::new(123, DENOM_UKUJI);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_denom_balance: Coin::new(122, DENOM_UKUJI),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        assert_eq!(
            return_swapped_funds(deps.as_ref(), mock_env()).unwrap_err(),
            ContractError::FailedSwap {
                msg: format!(
                    "{} is less than the minimum return amount of {}",
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

        let minimum_receive_amount = Coin::new(123, DENOM_UKUJI);
        let target_denom_balance = Coin::new(122, DENOM_UKUJI);
        let return_amount = Coin::new(153, DENOM_UKUJI);

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
