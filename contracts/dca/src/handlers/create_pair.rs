use crate::helpers::validation::assert_sender_is_admin;
use crate::state::pairs::save_pair;
use crate::{error::ContractError, types::pair::Pair};
use cosmwasm_std::{Addr, DepsMut};
use cosmwasm_std::{MessageInfo, Response};

pub fn create_pair_handler(
    deps: DepsMut,
    info: MessageInfo,
    base_denom: String,
    quote_denom: String,
    address: Addr,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;
    deps.api.addr_validate(address.as_ref())?;

    let pair = Pair {
        base_denom: base_denom.clone(),
        quote_denom: quote_denom.clone(),
        address: address.clone(),
    };

    save_pair(deps.storage, &pair)?;

    Ok(Response::new()
        .add_attribute("create_pair", "true")
        .add_attribute("base_denom", base_denom)
        .add_attribute("quote_denom", quote_denom)
        .add_attribute("address", format!("{:#?}", address)))
}

#[cfg(test)]
mod create_pair_tests {
    use crate::{
        contract::execute,
        msg::ExecuteMsg,
        state::pairs::find_pair,
        tests::{
            helpers::instantiate_contract,
            mocks::ADMIN,
            mocks::{DENOM_UKUJI, DENOM_UUSK},
        },
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    #[test]
    fn create_pair_that_already_exists_should_update_it() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let original_message = ExecuteMsg::CreatePair {
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair-1"),
        };

        let message = ExecuteMsg::CreatePair {
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair-2"),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), original_message).unwrap();

        let denoms = [DENOM_UKUJI.to_string(), DENOM_UUSK.to_string()];

        let original_pair = find_pair(deps.as_ref().storage, denoms.clone()).unwrap();

        execute(deps.as_mut(), env, info, message).unwrap();

        let pair = find_pair(deps.as_ref().storage, denoms).unwrap();

        assert_eq!(original_pair.address, Addr::unchecked("pair-1"));
        assert_eq!(pair.address, Addr::unchecked("pair-2"));
    }

    #[test]
    fn create_pair_with_unauthorised_sender_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let info_with_unauthorised_sender = mock_info("not-admin", &vec![]);

        let create_pair_execute_message = ExecuteMsg::CreatePair {
            base_denom: String::from("base"),
            quote_denom: String::from("quote"),
            address: Addr::unchecked("pair-1"),
        };

        let result = execute(
            deps.as_mut(),
            env,
            info_with_unauthorised_sender,
            create_pair_execute_message,
        )
        .unwrap_err();

        assert_eq!(result.to_string(), "Unauthorized")
    }

    #[test]
    fn recreate_pair_with_switched_denoms_should_overwrite_it() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let original_message = ExecuteMsg::CreatePair {
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair-1"),
        };

        let message = ExecuteMsg::CreatePair {
            quote_denom: DENOM_UKUJI.to_string(),
            base_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair-2"),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), original_message).unwrap();

        let denoms = [DENOM_UKUJI.to_string(), DENOM_UUSK.to_string()];

        let original_pair = find_pair(deps.as_ref().storage, denoms.clone()).unwrap();

        execute(deps.as_mut(), env, info, message).unwrap();

        let pair = find_pair(deps.as_ref().storage, denoms).unwrap();

        assert_eq!(original_pair.address, Addr::unchecked("pair-1"));
        assert_eq!(pair.address, Addr::unchecked("pair-2"));
    }
}
