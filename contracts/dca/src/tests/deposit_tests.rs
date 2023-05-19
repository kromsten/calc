use crate::constants::{ONE, ONE_HUNDRED, ONE_THOUSAND, TEN};
use crate::handlers::deposit::deposit_handler;
use crate::msg::{ExecuteMsg, QueryMsg, VaultResponse};
use crate::tests::mocks::{fin_contract_unfilled_limit_order, MockApp, ADMIN, DENOM_UKUJI, USER};
use crate::types::old_vault::OldVault;
use base::events::event::EventBuilder;
use base::vaults::vault::OldVaultStatus;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw_multi_test::Executor;

use super::helpers::{
    assert_address_balances, assert_events_published, assert_vault_balance, instantiate_contract,
    setup_new_vault,
};
use super::mocks::DENOM_UTEST;

#[test]
fn should_update_address_balances() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            None,
        );

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id: mock.vault_ids.get("vault").unwrap().to_owned(),
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::zero()),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );
}

#[test]
fn should_update_vault_balance() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        vault_deposit + vault_deposit,
    );
}

#[test]
fn should_create_event() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    assert_events_published(
        &mock,
        vault_id,
        &[EventBuilder::new(
            vault_id,
            mock.app.block_info(),
            base::events::event::EventData::DcaVaultFundsDeposited {
                amount: Coin::new(TEN.into(), DENOM_UKUJI),
            },
        )
        .build(2)],
    );
}

#[test]
fn when_vault_is_scheduled_should_not_change_status() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    assert_eq!(vault_response.vault.status, OldVaultStatus::Scheduled);
}

#[test]
fn when_vault_is_active_should_not_change_status() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_active_vault(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    assert_eq!(vault_response.vault.status, OldVaultStatus::Active);
}

#[test]
fn when_vault_is_active_should_not_execute_vault() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_active_vault(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    let initial_vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    assert_eq!(
        vault_response.vault.balance.amount,
        initial_vault_response.vault.balance.amount + vault_deposit
    );
}

#[test]
fn when_vault_is_inactive_should_change_status() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_inactive_vault(&user_address, None, "vault");

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    assert_eq!(vault_response.vault.status, OldVaultStatus::Active);
}

#[test]
fn when_vault_is_inactive_without_a_trigger_should_execute_vault() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

    let vault = setup_new_vault(
        deps.as_mut(),
        env.clone(),
        OldVault {
            status: OldVaultStatus::Inactive,
            trigger: None,
            ..OldVault::default()
        },
    );

    let response = deposit_handler(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[vault.balance]),
        Addr::unchecked(USER),
        vault.id,
    )
    .unwrap();

    assert!(response
        .messages
        .contains(&SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::ExecuteTrigger {
                trigger_id: vault.id
            })
            .unwrap(),
            funds: vec![]
        }))));
}

#[test]
fn when_vault_is_inactive_with_a_trigger_should_not_execute_vault() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &[]));

    let vault = setup_new_vault(
        deps.as_mut(),
        env.clone(),
        OldVault {
            status: OldVaultStatus::Inactive,
            ..OldVault::default()
        },
    );

    let response = deposit_handler(
        deps.as_mut(),
        env,
        mock_info(ADMIN, &[vault.balance]),
        Addr::unchecked(USER),
        vault.id,
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
}

#[test]
fn when_vault_is_inactive_and_insufficient_funds_should_not_change_status() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_inactive_vault(&user_address, None, "vault");

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(300, DENOM_UKUJI)],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetVault { vault_id })
        .unwrap();

    assert_eq!(vault_response.vault.status, OldVaultStatus::Inactive);
}

#[test]
fn when_vault_is_cancelled_should_fail() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_unfilled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(user_balance.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::CancelVault { vault_id },
            &[],
        )
        .unwrap();

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap_err();

    assert!(response
        .root_cause()
        .to_string()
        .contains("Error: vault is already cancelled"));
}

#[test]
fn with_multiple_assets_should_fail() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_unfilled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(user_balance.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
        );

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id: mock.vault_ids.get("vault").unwrap().to_owned(),
            },
            &[
                Coin::new(vault_deposit.into(), DENOM_UKUJI),
                Coin::new(vault_deposit.into(), DENOM_UTEST),
            ],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: received 2 denoms but required exactly 1"
    );
}

#[test]
fn with_mismatched_denom_should_fail() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_unfilled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(user_balance.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
        );

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id: mock.vault_ids.get("vault").unwrap().to_owned(),
            },
            &[Coin::new(vault_deposit.into(), DENOM_UTEST)],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: received asset with denom utest, but needed ukuji"
    );
}

#[test]
fn when_contract_is_paused_should_fail() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            None,
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::UpdateConfig {
                executors: None,
                fee_collectors: None,
                swap_fee_percent: None,
                delegation_fee_percent: None,
                staking_router_address: None,
                page_limit: None,
                paused: Some(true),
                dca_plus_escrow_level: None,
            },
            &[],
        )
        .unwrap();

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap_err();

    assert_eq!(
        "Error: contract is paused",
        response.root_cause().to_string()
    )
}

#[test]
fn with_dca_plus_should_update_model_id() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            Some(true),
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    let vault_before_deposit = mock
        .app
        .wrap()
        .query_wasm_smart::<VaultResponse>(
            &mock.dca_contract_address,
            &QueryMsg::GetVault { vault_id },
        )
        .unwrap()
        .vault;

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(
                (user_balance - vault_deposit).into(),
                DENOM_UKUJI,
            )],
        )
        .unwrap();

    let vault_after_deposit = mock
        .app
        .wrap()
        .query_wasm_smart::<VaultResponse>(
            &mock.dca_contract_address,
            &QueryMsg::GetVault { vault_id },
        )
        .unwrap()
        .vault;

    assert_eq!(vault_before_deposit.dca_plus_config.unwrap().model_id, 30);
    assert_eq!(vault_after_deposit.dca_plus_config.unwrap().model_id, 80);
}

#[test]
fn with_dca_plus_should_update_total_deposit() {
    let user_address = Addr::unchecked(USER);
    let user_balance = ONE_HUNDRED;
    let swap_amount = ONE;
    let vault_deposit = TEN;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "vault",
            None,
            Some(true),
        );

    let vault_id = mock.vault_ids.get("vault").unwrap().to_owned();

    let vault_before_deposit = mock
        .app
        .wrap()
        .query_wasm_smart::<VaultResponse>(
            &mock.dca_contract_address,
            &QueryMsg::GetVault { vault_id },
        )
        .unwrap()
        .vault;

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::Deposit {
                address: user_address.clone(),
                vault_id,
            },
            &[Coin::new(vault_deposit.into(), DENOM_UKUJI)],
        )
        .unwrap();

    let vault_after_deposit = mock
        .app
        .wrap()
        .query_wasm_smart::<VaultResponse>(
            &mock.dca_contract_address,
            &QueryMsg::GetVault { vault_id },
        )
        .unwrap()
        .vault;

    assert_eq!(
        vault_before_deposit
            .dca_plus_config
            .unwrap()
            .total_deposit
            .amount,
        vault_deposit
    );
    assert_eq!(
        vault_after_deposit
            .dca_plus_config
            .unwrap()
            .total_deposit
            .amount,
        vault_deposit + vault_deposit
    );
}
