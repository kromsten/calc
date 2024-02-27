use astrovault::assets::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    SubMsg, Uint128,
};
use cw_utils::one_coin;
use shared::cw20::is_cw20_token;

use crate::{
    contract::AFTER_SWAP,
    helpers::{balance::get_asset_balance, msg::send_asset_msg},
    state::{
        cache::{SwapCache, SWAP_CACHE},
        pairs::find_pair,
    },
    ContractError,
};

pub fn swap_native_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minimum_receive_amount: Asset,
    route: Option<Binary>,
) -> Result<Response, ContractError> {
    let coin = one_coin(&info)?;

    let asset = match is_cw20_token(deps.api, &coin.denom) {
        Ok(token_address) => Asset {
            info: AssetInfo::Token {
                contract_addr: token_address.to_string(),
            },
            amount: coin.amount,
        },
        Err(_) => Asset {
            info: AssetInfo::NativeToken { denom: coin.denom },
            amount: coin.amount,
        },
    };

    swap_handler(
        deps,
        env,
        info.sender,
        asset,
        minimum_receive_amount,
        info.funds,
        route,
    )
}

pub fn swap_msg(
    deps: Deps,
    env: Env,
    offer_asset: Asset,
    minimum_receive_amount: Asset,
    funds: Vec<Coin>,
    route: Option<Binary>,
) -> StdResult<CosmosMsg> {
    let pair = find_pair(
        deps.storage,
        [
            offer_asset.info.to_string(),
            minimum_receive_amount.info.to_string(),
        ],
    )?;

    pair.swap_msg(
        deps,
        env,
        offer_asset.clone(),
        minimum_receive_amount.clone(),
        route,
        funds,
    )
    .map_err(|e| StdError::generic_err(e.to_string()))
}

fn swap_handler(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    offer_asset: Asset,
    minimum_receive_amount: Asset,
    funds: Vec<Coin>,
    route: Option<Binary>,
) -> Result<Response, ContractError> {
    let pair = find_pair(
        deps.storage,
        [
            offer_asset.info.to_string(),
            minimum_receive_amount.info.to_string(),
        ],
    )?;

    SWAP_CACHE.save(
        deps.storage,
        &SwapCache {
            sender: sender.clone(),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_asset_balance: Asset {
                info: minimum_receive_amount.info.clone(),
                amount: get_asset_balance(
                    &deps.querier,
                    minimum_receive_amount.info.clone(),
                    env.contract.address.clone(),
                )?,
            },
        },
    )?;

    let swap_msgs = pair.swap_msg(
        deps.as_ref(),
        env,
        offer_asset.clone(),
        minimum_receive_amount.clone(),
        route,
        funds,
    )?;

    let sub_msg: SubMsg = SubMsg::reply_on_success(swap_msgs, AFTER_SWAP);

    Ok(Response::new()
        .add_attribute("swap", "true")
        .add_attribute("sender", sender)
        .add_attribute("swap_amount", offer_asset.amount.to_string())
        .add_attribute("minimum_receive_amount", minimum_receive_amount.to_string())
        .add_submessage(sub_msg))
}

pub fn return_swapped_funds(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    let updated_target_balance = get_asset_balance(
        &deps.querier,
        swap_cache.minimum_receive_amount.info.clone(),
        env.contract.address,
    )?;

    let return_amount = updated_target_balance
        .checked_sub(swap_cache.target_asset_balance.amount)
        .unwrap_or(Uint128::zero());

    if return_amount < swap_cache.minimum_receive_amount.amount {
        return Err(ContractError::FailedSwap {
            msg: format!(
                "{} is less than the minimum return amount of {}",
                return_amount, swap_cache.minimum_receive_amount
            ),
        });
    }

    let send_funds_msg = send_asset_msg(
        swap_cache.sender.to_string(),
        swap_cache.target_asset_balance.info,
        return_amount,
    )?;

    Ok(Response::new()
        .add_attribute("return_amount", return_amount.to_string())
        .add_message(send_funds_msg))
}

#[cfg(test)]
mod swap_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info},
        to_json_binary, Coin, StdError, SubMsg,
    };

    use astrovault::{
        assets::asset::{Asset, AssetInfo},
        standard_pool::handle_msg::ExecuteMsg,
    };
    use cw_utils::PaymentError;

    use crate::{
        contract::AFTER_SWAP,
        handlers::swap::swap_native_handler,
        helpers::balance::{asset_to_coin, coin_to_asset},
        state::{cache::SWAP_CACHE, pairs::save_pair},
        tests::constants::{ADMIN, DENOM_AARCH, DENOM_UUSDC},
        types::{pair::PopulatedPair, wrapper::ContractWrapper},
        ContractError,
    };

    #[test]
    fn with_no_assets_fails() {
        assert_eq!(
            swap_native_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[]),
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: DENOM_AARCH.into()
                    },
                    amount: 12313u128.into()
                },
                None
            )
            .unwrap_err(),
            ContractError::Payment(PaymentError::NoFunds {})
        )
    }

    #[test]
    fn with_multiple_assets_fails() {
        assert_eq!(
            swap_native_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(
                    ADMIN,
                    &[Coin::new(12312, DENOM_UUSDC), Coin::new(12312, DENOM_AARCH)]
                ),
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: DENOM_AARCH.into()
                    },
                    amount: 12312u128.into()
                },
                None
            )
            .unwrap_err(),
            ContractError::Payment(PaymentError::MultipleDenoms {})
        )
    }

    #[test]
    fn with_zero_swap_amount_fails() {
        assert_eq!(
            swap_native_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(0, DENOM_AARCH)]),
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: DENOM_AARCH.into()
                    },
                    amount: 12312u128.into()
                },
                None
            )
            .unwrap_err(),
            ContractError::Payment(PaymentError::NoFunds {})
        )
    }

    #[test]
    fn with_no_pair_fails() {
        let err = swap_native_handler(
            mock_dependencies().as_mut(),
            mock_env(),
            mock_info(ADMIN, &[Coin::new(12312, DENOM_AARCH)]),
            Asset {
                info: AssetInfo::NativeToken {
                    denom: DENOM_AARCH.into(),
                },
                amount: 12312u128.into(),
            },
            None,
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::Std(StdError::generic_err("Pair not found"))
        );
    }

    #[test]
    fn caches_details_correctly() {
        let mut deps = mock_dependencies_with_balance(&[Coin::new(0, DENOM_UUSDC)]);

        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom())]);
        let minimum_receive_amount = Coin::new(3873213, pair.base_denom());

        swap_native_handler(
            deps.as_mut(),
            mock_env(),
            info,
            coin_to_asset(minimum_receive_amount.clone()),
            None,
        )
        .unwrap();

        let swap_cache = SWAP_CACHE.load(deps.as_ref().storage).unwrap();

        assert_eq!(swap_cache.sender, ADMIN);
        assert_eq!(
            asset_to_coin(swap_cache.target_asset_balance),
            deps.as_ref()
                .querier
                .query_balance(
                    mock_env().contract.address,
                    minimum_receive_amount.denom.clone()
                )
                .unwrap()
        );
        assert_eq!(
            swap_cache.minimum_receive_amount,
            coin_to_asset(minimum_receive_amount)
        );
    }

    #[test]
    fn sends_swap_message() {
        let mut deps = mock_dependencies();

        let pair = PopulatedPair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, pair.quote_denom())]);

        let response = swap_native_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            Asset {
                info: AssetInfo::NativeToken {
                    denom: pair.base_denom(),
                },
                amount: 3873213u128.into(),
            },
            None,
        )
        .unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::reply_on_success(
                ContractWrapper(pair.pool().address)
                    .execute(
                        to_json_binary(&ExecuteMsg::Swap {
                            expected_return: Some(3873213u128.into()),
                            belief_price: None,
                            max_spread: None,
                            to: None,
                            offer_asset: Asset {
                                info: AssetInfo::NativeToken {
                                    denom: pair.quote_denom()
                                },
                                amount: 2347631u128.into()
                            },
                        })
                        .unwrap(),
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
        Addr, BankMsg, Coin, Uint128,
    };
    use shared::coin::add;

    use crate::{
        handlers::swap::return_swapped_funds,
        helpers::balance::coin_to_asset,
        state::cache::{SwapCache, SWAP_CACHE},
        tests::constants::DENOM_AARCH,
        ContractError,
    };

    #[test]
    fn with_return_amount_smaller_than_minimum_receive_amount_fails() {
        let mut deps = mock_dependencies();

        let minimum_receive_amount = Coin::new(123, DENOM_AARCH);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount: coin_to_asset(minimum_receive_amount.clone()),
            target_asset_balance: coin_to_asset(Coin::new(122, DENOM_AARCH)),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        assert_eq!(
            return_swapped_funds(deps.as_ref(), mock_env()).unwrap_err(),
            ContractError::FailedSwap {
                msg: format!(
                    "{} is less than the minimum return amount of {}",
                    Uint128::zero(),
                    minimum_receive_amount
                )
            }
        )
    }

    #[test]
    fn sends_funds_back_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let minimum_receive_amount = Coin::new(123, DENOM_AARCH);
        let target_denom_balance = Coin::new(122, DENOM_AARCH);
        let return_amount = Coin::new(153, DENOM_AARCH);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount: coin_to_asset(minimum_receive_amount.clone()),
            target_asset_balance: coin_to_asset(target_denom_balance.clone()),
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
