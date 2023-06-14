use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::ContractError;

pub fn withdraw_order_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn return_withdrawn_funds(_deps: Deps, _env: Env) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod withdraw_order_handler_tests {
    use std::{collections::HashMap, vec};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin, Uint128,
    };

    use crate::{
        handlers::withdraw_order::withdraw_order_handler,
        state::{cache::LIMIT_ORDER_CACHE, pairs::save_pair},
        tests::constants::{ADMIN, DENOM_UATOM, DENOM_UOSMO},
        types::pair::Pair,
        ContractError,
    };

    #[test]
    fn with_funds_fails() {
        assert_eq!(
            withdraw_order_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(3218312, DENOM_UATOM)]),
                Uint128::new(234),
                [DENOM_UATOM.to_string(), DENOM_UOSMO.to_string()],
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

        let uusk_balance = Coin::new(25423, DENOM_UATOM);
        let ukuji_balance = Coin::new(12234324343123, DENOM_UOSMO);

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
            [DENOM_UATOM.to_string(), DENOM_UOSMO.to_string()],
        )
        .unwrap();

        let cache = LIMIT_ORDER_CACHE.load(deps.as_ref().storage).unwrap();

        assert_eq!(cache.sender, ADMIN.to_string());
        assert_eq!(
            cache.balances,
            HashMap::from([
                (DENOM_UATOM.to_string(), uusk_balance),
                (DENOM_UOSMO.to_string(), ukuji_balance)
            ])
        );
    }

    #[test]
    fn sends_withdraw_order_message() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order_idx = Uint128::new(182374);

        let response = withdraw_order_handler(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            order_idx,
            [DENOM_UATOM.to_string(), DENOM_UOSMO.to_string()],
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        // assert_eq!(
        //     response.messages.first().unwrap(),
        //     &SubMsg::reply_on_success(
        //         PairContract(pair.address)
        //             .call(
        //                 ExecuteMsg::WithdrawOrders {
        //                     order_idxs: Some(vec![order_idx]),
        //                     callback: None
        //                 },
        //                 vec![],
        //             )
        //             .unwrap(),
        //         AFTER_WITHDRAW_ORDER,
        //     )
        // );
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
        state::cache::{LimitOrderCache, LIMIT_ORDER_CACHE},
        tests::constants::{ADMIN, DENOM_UATOM, DENOM_UOSMO},
    };

    #[test]
    fn returns_funds_difference_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let old_uusk_balance = Coin::new(25423, DENOM_UATOM);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UOSMO);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UATOM.to_string(), old_uusk_balance.clone()),
                        (DENOM_UOSMO.to_string(), old_ukuji_balance.clone()),
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
                amount: vec![Coin::new(1000, DENOM_UATOM), Coin::new(2000, DENOM_UOSMO)],
            })
        );
    }

    #[test]
    fn drops_empty_funds_differences() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let old_uusk_balance = Coin::new(25423, DENOM_UATOM);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UOSMO);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UATOM.to_string(), old_uusk_balance.clone()),
                        (DENOM_UOSMO.to_string(), old_ukuji_balance.clone()),
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
                amount: vec![Coin::new(2000, DENOM_UOSMO)],
            })
        );
    }

    #[test]
    fn with_no_differences_drops_bank_send_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let old_uusk_balance = Coin::new(25423, DENOM_UATOM);
        let old_ukuji_balance = Coin::new(12234324343123, DENOM_UOSMO);

        LIMIT_ORDER_CACHE
            .save(
                deps.as_mut().storage,
                &LimitOrderCache {
                    sender: Addr::unchecked(ADMIN),
                    balances: HashMap::from([
                        (DENOM_UATOM.to_string(), old_uusk_balance.clone()),
                        (DENOM_UOSMO.to_string(), old_ukuji_balance.clone()),
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
