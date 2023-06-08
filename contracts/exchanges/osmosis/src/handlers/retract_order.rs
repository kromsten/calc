use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::ContractError;

pub fn retract_order_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _order_idx: Uint128,
    _denoms: [String; 2],
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn return_retracted_funds(_deps: Deps, _env: Env) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod retract_order_handler_tests {
    use std::{collections::HashMap, vec};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin, Uint128,
    };

    use crate::{
        state::{cache::LIMIT_ORDER_CACHE, pairs::save_pair},
        tests::constants::{ADMIN, DENOM_UKUJI, DENOM_UUSK},
        types::pair::Pair,
        ContractError,
    };

    use super::retract_order_handler;

    #[test]
    fn with_funds_fails() {
        assert_eq!(
            retract_order_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(3218312, DENOM_UUSK)]),
                Uint128::new(234),
                [DENOM_UUSK.to_string(), DENOM_UKUJI.to_string()],
            )
            .unwrap_err(),
            ContractError::InvalidFunds {
                msg: String::from("must not provide funds to retract order")
            }
        );
    }

    #[test]
    fn caches_sender_and_pair_balances() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let uusk_balance = Coin::new(25423, DENOM_UUSK);
        let ukuji_balance = Coin::new(12234324343123, DENOM_UKUJI);

        let balances = vec![uusk_balance.clone(), ukuji_balance.clone()];

        deps.querier
            .update_balance(env.contract.address.clone(), balances);

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order_idx = Uint128::new(182374);

        retract_order_handler(
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

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let order_idx = Uint128::new(182374);

        let response = retract_order_handler(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            order_idx,
            [DENOM_UUSK.to_string(), DENOM_UKUJI.to_string()],
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        // assert_eq!(
        //     response.messages.first().unwrap(),
        //     &SubMsg::reply_on_success(
        //         SubMsg::,
        //         AFTER_RETRACT_ORDER,
        //     )
        // );
    }
}

#[cfg(test)]
mod return_retracted_funds_tests {
    use std::collections::HashMap;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, BankMsg, Coin, SubMsg, Uint128,
    };
    use shared::coin::add_to;

    use crate::{
        state::cache::{LimitOrderCache, LIMIT_ORDER_CACHE},
        tests::constants::{ADMIN, DENOM_UKUJI, DENOM_UUSK},
    };

    use super::return_retracted_funds;

    #[test]
    fn returns_funds_difference_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

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

        let response = return_retracted_funds(deps.as_ref(), env).unwrap();

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

        let response = return_retracted_funds(deps.as_ref(), env).unwrap();

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

        let response = return_retracted_funds(deps.as_ref(), env).unwrap();

        assert_eq!(response.messages.len(), 0);
    }
}
