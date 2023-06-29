use std::collections::HashMap;

use cosmwasm_std::{BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use kujira_fin::ExecuteMsg;

use crate::{
    contract::AFTER_WITHDRAW_ORDER,
    helpers::balance::get_balance_delta,
    state::{
        cache::{LimitOrderCache, LIMIT_ORDER_CACHE},
        config::get_config,
        pairs::find_pair,
    },
    types::pair_contract::PairContract,
    ContractError,
};

pub fn withdraw_order_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_idx: Uint128,
    denoms: [String; 2],
) -> Result<Response, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::InvalidFunds {
            msg: "must not provide funds to withdraw order".to_string(),
        });
    }

    let config = get_config(deps.storage)?;

    if info.sender != config.dca_contract_address {
        return Err(ContractError::Unauthorized {});
    }

    LIMIT_ORDER_CACHE.save(
        deps.storage,
        &LimitOrderCache {
            sender: info.sender,
            balances: HashMap::from([
                (
                    denoms[0].clone(),
                    deps.querier
                        .query_balance(env.contract.address.clone(), denoms[0].clone())?,
                ),
                (
                    denoms[1].clone(),
                    deps.querier
                        .query_balance(env.contract.address, denoms[1].clone())?,
                ),
            ]),
        },
    )?;

    let pair = find_pair(deps.storage, denoms)?;

    Ok(Response::new()
        .add_attribute("withdraw_order", "true")
        .add_attribute("fin_pair", pair.address.clone())
        .add_attribute("order_idx", order_idx)
        .add_submessage(SubMsg::reply_on_success(
            PairContract(pair.address).call(
                ExecuteMsg::WithdrawOrders {
                    order_idxs: Some(vec![order_idx]),
                    callback: None,
                },
                vec![],
            )?,
            AFTER_WITHDRAW_ORDER,
        )))
}

pub fn return_withdrawn_funds(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let cache = LIMIT_ORDER_CACHE.load(deps.storage)?;

    let mut funds = cache
        .balances
        .values()
        .map(|old_balance| {
            get_balance_delta(deps.querier, env.contract.address.clone(), old_balance)
        })
        .collect::<Result<Vec<Coin>, _>>()?
        .into_iter()
        .filter(|coin| !coin.amount.is_zero())
        .collect::<Vec<Coin>>();

    let mut response = Response::new().add_attribute("return_withdrawn_funds", "true");

    if !funds.is_empty() {
        funds.sort_by(|a, b| a.amount.cmp(&b.amount));
        response = response.add_submessage(SubMsg::new(BankMsg::Send {
            to_address: cache.sender.to_string(),
            amount: funds,
        }));
    }

    Ok(response)
}

#[cfg(test)]
mod withdraw_order_handler_tests {
    use std::{collections::HashMap, vec};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Coin, SubMsg, Uint128,
    };
    use kujira_fin::ExecuteMsg;

    use crate::{
        contract::AFTER_WITHDRAW_ORDER,
        handlers::withdraw_order::withdraw_order_handler,
        state::{cache::LIMIT_ORDER_CACHE, config::update_config, pairs::save_pair},
        tests::constants::{ADMIN, DCA_CONTRACT, DENOM_UKUJI, DENOM_UUSK},
        types::{config::Config, pair::Pair, pair_contract::PairContract},
        ContractError,
    };

    #[test]
    fn with_funds_fails() {
        let mut deps = mock_dependencies();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        assert_eq!(
            withdraw_order_handler(
                deps.as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(3218312, DENOM_UUSK)]),
                Uint128::new(234),
                [DENOM_UUSK.to_string(), DENOM_UKUJI.to_string()],
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: String::from("must not provide funds to withdraw order")
            }
        );
    }

    #[test]
    fn caches_sender_and_pair_balances() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        let uusk_balance = Coin::new(25423, DENOM_UUSK);
        let ukuji_balance = Coin::new(12234324343123, DENOM_UKUJI);

        let balances = vec![uusk_balance.clone(), ukuji_balance.clone()];

        deps.querier
            .update_balance(env.contract.address.clone(), balances);

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order_idx = Uint128::new(182374);

        withdraw_order_handler(
            deps.as_mut(),
            env,
            mock_info(ADMIN, &[]),
            order_idx,
            [DENOM_UUSK.to_string(), DENOM_UKUJI.to_string()],
        )
        .unwrap();

        let cache = LIMIT_ORDER_CACHE.load(deps.as_ref().storage).unwrap();

        assert_eq!(cache.sender, ADMIN.to_string());
        assert_eq!(
            cache.balances,
            HashMap::from([
                (DENOM_UUSK.to_string(), uusk_balance),
                (DENOM_UKUJI.to_string(), ukuji_balance)
            ])
        );
    }

    #[test]
    fn sends_withdraw_order_message() {
        let mut deps = mock_dependencies();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order_idx = Uint128::new(182374);

        let response = withdraw_order_handler(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            order_idx,
            [DENOM_UUSK.to_string(), DENOM_UKUJI.to_string()],
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::reply_on_success(
                PairContract(pair.address)
                    .call(
                        ExecuteMsg::WithdrawOrders {
                            order_idxs: Some(vec![order_idx]),
                            callback: None
                        },
                        vec![],
                    )
                    .unwrap(),
                AFTER_WITHDRAW_ORDER,
            )
        );
    }
}

#[cfg(test)]
mod return_withdrawn_funds_tests {
    use std::collections::HashMap;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, BankMsg, Coin, SubMsg, Uint128,
    };
    use shared::coin::add_to;

    use crate::{
        handlers::withdraw_order::return_withdrawn_funds,
        state::{
            cache::{LimitOrderCache, LIMIT_ORDER_CACHE},
            config::update_config,
        },
        tests::constants::{ADMIN, DCA_CONTRACT, DENOM_UKUJI, DENOM_UUSK},
        types::config::Config,
    };

    #[test]
    fn returns_funds_difference_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        let old_uusk_balance = Coin::new(25423, DENOM_UUSK);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UKUJI);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UUSK.to_string(), old_uusk_balance.clone()),
                        (DENOM_UKUJI.to_string(), old_ukuji_balance.clone()),
                    ]),
                },
            )
            .unwrap();

        let new_uusk_balance = add_to(&old_uusk_balance, Uint128::new(1000));
        let new_ukuji_balance = add_to(&old_ukuji_balance, Uint128::new(2000));

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![new_uusk_balance, new_ukuji_balance],
        );

        let response = return_withdrawn_funds(deps.as_ref(), env).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::new(BankMsg::Send {
                to_address: ADMIN.to_string(),
                amount: vec![Coin::new(1000, DENOM_UUSK), Coin::new(2000, DENOM_UKUJI)],
            })
        );
    }

    #[test]
    fn drops_empty_funds_differences() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        let old_uusk_balance = Coin::new(25423, DENOM_UUSK);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UKUJI);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UUSK.to_string(), old_uusk_balance.clone()),
                        (DENOM_UKUJI.to_string(), old_ukuji_balance.clone()),
                    ]),
                },
            )
            .unwrap();

        let new_ukuji_balance = add_to(&old_ukuji_balance, Uint128::new(2000));

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![old_uusk_balance, new_ukuji_balance],
        );

        let response = return_withdrawn_funds(deps.as_ref(), env).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::new(BankMsg::Send {
                to_address: ADMIN.to_string(),
                amount: vec![Coin::new(2000, DENOM_UKUJI)],
            })
        );
    }

    #[test]
    fn with_no_differences_drops_bank_send_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                dca_contract_address: Addr::unchecked(DCA_CONTRACT),
            },
        )
        .unwrap();

        let old_uusk_balance = Coin::new(25423, DENOM_UUSK);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UKUJI);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UUSK.to_string(), old_uusk_balance.clone()),
                        (DENOM_UKUJI.to_string(), old_ukuji_balance.clone()),
                    ]),
                },
            )
            .unwrap();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![old_uusk_balance, old_ukuji_balance],
        );

        let response = return_withdrawn_funds(deps.as_ref(), env).unwrap();

        assert_eq!(response.messages.len(), 0);
    }
}
