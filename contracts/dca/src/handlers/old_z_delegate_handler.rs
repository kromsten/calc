use crate::{
    constants::AFTER_DELEGATION_REPLY_ID,
    error::ContractError,
    helpers::validation::{
        assert_address_is_valid, assert_exactly_one_asset, assert_validator_is_valid,
    },
    state::config::get_config,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Deps, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use std::vec;

#[cw_serde]
pub enum StakingRouterExecuteMsg {
    ZDelegate {
        delegator_address: Addr,
        validator_address: Addr,
        amount: Uint128,
        denom: String,
    },
}

pub fn old_z_delegate_handler(
    deps: Deps,
    info: MessageInfo,
    delegator_address: Addr,
    validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;
    assert_address_is_valid(deps, &delegator_address, "delegator address")?;
    assert_validator_is_valid(deps, validator_address.to_string())?;

    let amount_to_delegate = info.funds[0].clone();

    let config = get_config(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("z_delegate", "true".to_string()),
            ("delegation", amount_to_delegate.to_string()),
            ("delegator", delegator_address.to_string()),
            ("validator", validator_address.to_string()),
        ])
        .add_submessages(vec![
            SubMsg::new(BankMsg::Send {
                to_address: delegator_address.to_string(),
                amount: vec![amount_to_delegate.clone()],
            }),
            SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: config.old_staking_router_address.to_string(),
                    msg: to_binary(&StakingRouterExecuteMsg::ZDelegate {
                        delegator_address,
                        validator_address,
                        amount: amount_to_delegate.amount,
                        denom: amount_to_delegate.denom,
                    })
                    .unwrap(),
                    funds: vec![],
                },
                AFTER_DELEGATION_REPLY_ID,
            ),
        ]))
}

#[cfg(test)]
mod old_z_delegate_handler_tests {
    use super::old_z_delegate_handler;
    use crate::{
        constants::AFTER_DELEGATION_REPLY_ID,
        handlers::old_z_delegate_handler::StakingRouterExecuteMsg,
        state::config::get_config,
        tests::{
            helpers::instantiate_contract,
            mocks::{ADMIN, DENOM_UKUJI},
        },
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, Addr, BankMsg, Coin, SubMsg,
    };

    #[test]
    fn with_more_than_one_asset_fails() {
        let err = old_z_delegate_handler(
            mock_dependencies().as_ref(),
            mock_info(
                ADMIN,
                &[Coin::new(213312, "asdasd"), Coin::new(234322, "asddua")],
            ),
            Addr::unchecked("delegator"),
            Addr::unchecked("validator"),
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: received 2 denoms but required exactly 1"
        );
    }

    #[test]
    fn with_no_assets_sent_fails() {
        let err = old_z_delegate_handler(
            mock_dependencies().as_ref(),
            mock_info(ADMIN, &[]),
            Addr::unchecked("delegator"),
            Addr::unchecked("validator"),
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Error: received 0 denoms but required exactly 1"
        );
    }

    #[test]
    fn sends_bank_message() {
        let mut deps = mock_dependencies();
        let info = mock_info(ADMIN, &[Coin::new(213312, DENOM_UKUJI)]);

        instantiate_contract(deps.as_mut(), mock_env(), info.clone());

        let delegator_address = Addr::unchecked("delegator");

        let response = old_z_delegate_handler(
            deps.as_ref(),
            info,
            delegator_address.clone(),
            Addr::unchecked("validator"),
        )
        .unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg::new(BankMsg::Send {
                to_address: delegator_address.to_string(),
                amount: vec![Coin::new(213312, DENOM_UKUJI)],
            })
        )
    }

    #[test]
    fn sends_z_delegate_message() {
        let mut deps = mock_dependencies();
        let info = mock_info(ADMIN, &[Coin::new(213312, DENOM_UKUJI)]);

        instantiate_contract(deps.as_mut(), mock_env(), info.clone());

        let delegator_address = Addr::unchecked("delegator");
        let validator_address = Addr::unchecked("validator");

        let response = old_z_delegate_handler(
            deps.as_ref(),
            info.clone(),
            delegator_address.clone(),
            validator_address.clone(),
        )
        .unwrap();

        let config = get_config(deps.as_ref().storage).unwrap();

        assert_eq!(
            response.messages.last().unwrap(),
            &SubMsg::reply_always(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: config.old_staking_router_address.to_string(),
                    msg: to_binary(&StakingRouterExecuteMsg::ZDelegate {
                        delegator_address,
                        validator_address,
                        amount: info.funds[0].amount,
                        denom: info.funds[0].denom.clone(),
                    })
                    .unwrap(),
                    funds: vec![],
                },
                AFTER_DELEGATION_REPLY_ID,
            )
        )
    }
}
