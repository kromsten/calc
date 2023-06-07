use super::math::checked_mul;
use crate::{
    state::config::get_config,
    types::{
        fee_collector::FeeCollector,
        performance_assessment_strategy::PerformanceAssessmentStrategy,
        swap_adjustment_strategy::SwapAdjustmentStrategy, vault::Vault,
    },
};
use cosmos_sdk_proto::{
    cosmos::base::v1beta1::Coin as ProtoCoin, cosmos::distribution::v1beta1::MsgFundCommunityPool,
    traits::Message,
};
use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, Env, StdResult, Storage, SubMsg, Uint128,
};
use std::cmp::min;

pub fn get_fee_messages(
    deps: Deps,
    env: Env,
    fee_amounts: Vec<Uint128>,
    denom: String,
    skip_community_pool: bool,
) -> StdResult<Vec<SubMsg>> {
    let config = get_config(deps.storage)?;

    let fee_collectors = config
        .fee_collectors
        .iter()
        .flat_map(|fee_collector| {
            if skip_community_pool && fee_collector.address == "community_pool" {
                return None;
            }
            return Some(FeeCollector {
                address: fee_collector.address.clone(),
                allocation: if skip_community_pool {
                    let community_pool_allocation = config
                        .fee_collectors
                        .iter()
                        .find(|fee_collector| fee_collector.address == "community_pool")
                        .map_or(Decimal::zero(), |community_pool| community_pool.allocation);
                    fee_collector.allocation / (Decimal::one() - community_pool_allocation)
                } else {
                    fee_collector.allocation
                },
            });
        })
        .collect::<Vec<FeeCollector>>();

    Ok(fee_collectors
        .iter()
        .flat_map(|fee_collector| {
            fee_amounts.iter().flat_map(|fee| {
                let fee_allocation = Coin::new(
                    checked_mul(*fee, fee_collector.allocation)
                        .expect("amount to be distributed should be valid")
                        .into(),
                    denom.clone(),
                );

                if fee_allocation.amount.gt(&Uint128::zero()) {
                    match fee_collector.address.as_str() {
                        "community_pool" => {
                            if skip_community_pool {
                                None
                            } else {
                                Some(SubMsg::new(CosmosMsg::Stargate {
                                    type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool"
                                        .to_string(),
                                    value: Binary::from(
                                        MsgFundCommunityPool {
                                            amount: vec![ProtoCoin {
                                                denom: fee_allocation.denom.clone(),
                                                amount: fee_allocation.amount.to_string(),
                                            }],
                                            depositor: env.contract.address.to_string(),
                                        }
                                        .encode_to_vec(),
                                    ),
                                }))
                            }
                        }
                        _ => Some(SubMsg::new(BankMsg::Send {
                            to_address: fee_collector.address.to_string(),
                            amount: vec![fee_allocation],
                        })),
                    }
                } else {
                    None
                }
            })
        })
        .collect::<Vec<SubMsg>>())
}

pub fn get_automation_fee_rate(storage: &dyn Storage, vault: &Vault) -> StdResult<Decimal> {
    let default_automation_fee_level = get_config(storage)?.automation_fee_percent;

    Ok(default_automation_fee_level.checked_mul(
        vault
            .destinations
            .iter()
            .filter(|destination| destination.msg.is_some())
            .map(|destination| destination.allocation)
            .sum(),
    )?)
}

pub fn get_swap_fee_rate(
    storage: &dyn Storage,
    swap_adjustment_strategy: &Option<SwapAdjustmentStrategy>,
) -> StdResult<Decimal> {
    let config = get_config(storage)?;

    Ok(match swap_adjustment_strategy {
        Some(SwapAdjustmentStrategy::WeightedScale { .. }) => {
            config.weighted_scale_swap_fee_percent
        }
        Some(_) => Decimal::zero(),
        None => config.default_swap_fee_percent,
    })
}

pub fn get_performance_fee(vault: &Vault, current_price: Decimal) -> StdResult<Coin> {
    Ok(vault.performance_assessment_strategy.clone().map_or(
        Coin::new(0, vault.target_denom.clone()),
        |strategy| match strategy.clone() {
            PerformanceAssessmentStrategy::CompareToStandardDca {
                swapped_amount,
                received_amount,
            } => {
                let vault_total_value = vault.deposited_amount.amount - vault.swapped_amount.amount
                    + vault.received_amount.amount * current_price;

                let standard_dca_total_value = vault.deposited_amount.amount
                    - swapped_amount.amount
                    + received_amount.amount * current_price;

                let added_value_in_terms_of_receive_denom = vault_total_value
                    .checked_sub(standard_dca_total_value)
                    .unwrap_or(Uint128::zero())
                    * (Decimal::one() / current_price);

                Coin {
                    denom: vault.target_denom.clone(),
                    amount: min(
                        vault.escrowed_amount.amount,
                        added_value_in_terms_of_receive_denom * strategy.performance_fee_rate(),
                    ),
                }
            }
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::get_swap_fee_rate;
    use crate::{
        constants::{ONE, TEN},
        helpers::fees::get_performance_fee,
        state::config::get_config,
        tests::{helpers::instantiate_contract, mocks::ADMIN},
        types::{
            performance_assessment_strategy::PerformanceAssessmentStrategy,
            swap_adjustment_strategy::SwapAdjustmentStrategy, vault::Vault,
        },
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin, Decimal, Uint128,
    };
    use std::str::FromStr;

    fn get_vault(
        total_deposit: Uint128,
        swapped_amount: Uint128,
        standard_dca_swapped_amount: Uint128,
        received_amount: Uint128,
        standard_dca_received_amount: Uint128,
    ) -> Vault {
        let escrow_level = Decimal::percent(5);

        Vault {
            balance: Coin::new(
                (total_deposit - swapped_amount).into(),
                "swap_denom".to_string(),
            ),
            deposited_amount: Coin::new(total_deposit.into(), "swap_denom".to_string()),
            escrow_level,
            swapped_amount: Coin::new(swapped_amount.into(), "swap_denom".to_string()),
            received_amount: Coin::new(received_amount.into(), "receive_denom".to_string()),
            escrowed_amount: Coin::new(
                (received_amount * escrow_level).into(),
                "denom".to_string(),
            ),
            performance_assessment_strategy: Some(
                PerformanceAssessmentStrategy::CompareToStandardDca {
                    swapped_amount: Coin::new(
                        standard_dca_swapped_amount.into(),
                        "swap_denom".to_string(),
                    ),
                    received_amount: Coin::new(
                        standard_dca_received_amount.into(),
                        "receive_denom".to_string(),
                    ),
                },
            ),
            swap_adjustment_strategy: Some(SwapAdjustmentStrategy::default()),
            ..Vault::default()
        }
    }

    fn assert_fee_amount(
        total_deposit: Uint128,
        swapped_amount: Uint128,
        standard_dca_swapped_amount: Uint128,
        received_amount: Uint128,
        standard_dca_received_amount: Uint128,
        current_price: Decimal,
        expected_fee: Uint128,
    ) {
        let vault = get_vault(
            total_deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
        );

        let fee = get_performance_fee(&vault, current_price).unwrap();
        assert_eq!(fee.amount, expected_fee);
    }

    #[test]
    fn non_zero_fee_is_in_vault_receive_denom() {
        let vault = get_vault(TEN, TEN, TEN, TEN + TEN, TEN);

        let fee = get_performance_fee(&vault, Decimal::one()).unwrap();
        assert_eq!(fee.denom, vault.target_denom);
    }

    #[test]
    fn zero_fee_is_in_vault_receive_denom() {
        let vault = get_vault(TEN, TEN, TEN, TEN, TEN);

        let fee = get_performance_fee(&vault, Decimal::one()).unwrap();
        assert_eq!(fee.denom, vault.target_denom);
    }

    #[test]
    fn fee_is_zero_when_performance_is_even() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(1000);
        let received_amount = Uint128::new(1000);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("1.0").unwrap();
        let expected_fee = Uint128::new(0);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_above_zero_when_less_swapped_and_price_dropped() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(900);
        let received_amount = Uint128::new(900);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("0.9").unwrap();
        let expected_fee = Uint128::new(2);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_equal_to_escrow_when_less_swapped_and_price_dropped_significantly() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(900);
        let received_amount = Uint128::new(1000);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("0.2").unwrap();
        let expected_fee = Uint128::new(50);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_zero_when_more_swapped_and_price_dropped() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(1100);
        let received_amount = Uint128::new(1000);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("0.9").unwrap();
        let expected_fee = Uint128::new(0);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_above_zero_when_more_swapped_and_price_increased() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(1100);
        let received_amount = Uint128::new(1100);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("2").unwrap();
        let expected_fee = Uint128::new(10);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_equal_to_escrow_when_same_amount_swapped_and_more_received() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(1000);
        let received_amount = Uint128::new(2000);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("1").unwrap();
        let expected_fee = Uint128::new(100);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn fee_is_zero_when_less_swapped_and_price_increased() {
        let deposit = Uint128::new(2000);
        let swapped_amount = Uint128::new(900);
        let received_amount = Uint128::new(900);
        let standard_dca_swapped_amount = Uint128::new(1000);
        let standard_dca_received_amount = Uint128::new(1000);
        let current_price = Decimal::from_str("1.1").unwrap();
        let expected_fee = Uint128::new(0);

        assert_fee_amount(
            deposit,
            swapped_amount,
            standard_dca_swapped_amount,
            received_amount,
            standard_dca_received_amount,
            current_price,
            expected_fee,
        );
    }

    #[test]
    fn swap_adjustment_specific_fee_level_is_used() {
        let mut deps = mock_dependencies();

        instantiate_contract(deps.as_mut(), mock_env(), mock_info(ADMIN, &[]));

        let swap_adjustment_strategy = SwapAdjustmentStrategy::WeightedScale {
            base_receive_amount: ONE,
            multiplier: Decimal::one(),
            increase_only: false,
        };

        let vault = Vault {
            swap_adjustment_strategy: Some(swap_adjustment_strategy),
            ..Default::default()
        };

        let fee_rate =
            get_swap_fee_rate(deps.as_ref().storage, &vault.swap_adjustment_strategy).unwrap();

        let config = get_config(deps.as_ref().storage).unwrap();

        assert_eq!(config.weighted_scale_swap_fee_percent, fee_rate);
    }
}
