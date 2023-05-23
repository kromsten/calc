use crate::types::{pair::Pair, position_type::PositionType, vault::Vault};
use cosmwasm_std::{Coin, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
use kujira::{
    asset::{Asset, AssetInfo},
    denom::Denom,
    fin::QueryMsg,
    precision::{Precise, Precision},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinConfigResponse {
    pub price_precision: Precision,
    pub decimal_delta: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FinPoolResponse {
    pub quote_price: Decimal,
    pub total_offer_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FinBookResponse {
    pub base: Vec<FinPoolResponse>,
    pub quote: Vec<FinPoolResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FinSimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
}

pub fn query_belief_price(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_denom: String,
) -> StdResult<Decimal> {
    let position_type = match swap_denom == pair.quote_denom {
        true => PositionType::Enter,
        false => PositionType::Exit,
    };

    let book_response = querier.query_wasm_smart::<FinBookResponse>(
        pair.address.clone(),
        &QueryMsg::Book {
            limit: Some(1),
            offset: None,
        },
    )?;

    let book = match position_type {
        PositionType::Enter => book_response.base,
        PositionType::Exit => book_response.quote,
    };

    if book.is_empty() {
        return Err(StdError::generic_err(format!(
            "No orders found for pair {:?}",
            pair
        )));
    }

    Ok(book[0].quote_price)
}

pub fn simulate_swap(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_amount: &Coin,
) -> StdResult<FinSimulationResponse> {
    querier.query_wasm_smart::<FinSimulationResponse>(
        pair.address.clone(),
        &QueryMsg::Simulation {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: Denom::from(swap_amount.denom.clone()),
                },
                amount: swap_amount.amount,
            },
        },
    )
}

pub fn query_price(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_amount: &Coin,
) -> StdResult<Decimal> {
    let simulation = simulate_swap(querier, pair, swap_amount)?;

    Ok(Decimal::from_ratio(
        swap_amount.amount,
        simulation.return_amount,
    ))
}

pub fn get_target_price(
    querier: &QuerierWrapper,
    vault: &Vault,
    pair: &Pair,
    target_receive_amount: Uint128,
) -> StdResult<Decimal> {
    let pair_config = querier
        .query_wasm_smart::<FinConfigResponse>(pair.address.clone(), &QueryMsg::Config {})?;

    if pair_config.decimal_delta < 0 {
        return Err(StdError::GenericErr {
            msg: "Negative decimal deltas are not supported".to_string(),
        });
    }

    calculate_target_price(
        vault,
        pair,
        target_receive_amount,
        pair_config.decimal_delta,
        pair_config.price_precision,
    )
}

fn calculate_target_price(
    vault: &Vault,
    pair: &Pair,
    target_receive_amount: Uint128,
    decimal_delta: i8,
    precision: Precision,
) -> StdResult<Decimal> {
    let exact_target_price = match pair.position_type(vault.get_swap_denom()) {
        PositionType::Enter => Decimal::from_ratio(vault.swap_amount, target_receive_amount),
        PositionType::Exit => Decimal::from_ratio(target_receive_amount, vault.swap_amount),
    };

    if decimal_delta == 0 {
        return Ok(exact_target_price.round(&precision));
    }

    let adjustment =
        Decimal::from_str(&10u128.pow(decimal_delta.unsigned_abs() as u32).to_string()).unwrap();

    let rounded_price = exact_target_price
        .checked_mul(adjustment)
        .unwrap()
        .round(&precision);

    Ok(rounded_price.checked_div(adjustment).unwrap())
}

#[cfg(test)]
mod calculate_target_price_tests {
    use super::*;

    #[test]
    fn should_be_correct_when_buying_on_fin() {
        let pair = Pair::default();

        let vault = Vault {
            swap_amount: Uint128::new(100),
            target_denom: pair.base_denom.clone(),
            balance: Coin::new(100, pair.quote_denom.clone()),
            ..Vault::default()
        };

        assert_eq!(
            calculate_target_price(
                &vault,
                &pair,
                Uint128::new(20),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "5"
        );
    }

    #[test]
    fn should_be_correct_when_selling_on_fin() {
        let pair = Pair::default();

        let vault = Vault {
            swap_amount: Uint128::new(100),
            target_denom: pair.quote_denom.clone(),
            balance: Coin::new(100, pair.base_denom.clone()),
            ..Vault::default()
        };

        assert_eq!(
            calculate_target_price(
                &vault,
                &pair,
                Uint128::new(20),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "0.2"
        );
    }

    #[test]
    fn should_truncate_price_to_three_decimal_places() {
        let pair = Pair::default();

        let vault = Vault {
            swap_amount: Uint128::new(30),
            target_denom: pair.quote_denom.clone(),
            balance: Coin::new(100, pair.base_denom.clone()),
            ..Vault::default()
        };

        assert_eq!(
            calculate_target_price(
                &vault,
                &pair,
                Uint128::new(10),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "0.333"
        );
    }

    #[test]
    fn for_fin_buy_with_decimal_delta_should_truncate() {
        let swap_amount = Uint128::new(1000000);
        let target_receive_amount = Uint128::new(747943156999999);
        let decimal_delta = 12;
        let precision = Precision::DecimalPlaces(2);

        let pair = Pair::default();

        let vault = Vault {
            swap_amount: swap_amount,
            target_denom: pair.base_denom.clone(),
            balance: Coin::new(100, pair.quote_denom.clone()),
            ..Vault::default()
        };

        assert_eq!(
            Decimal::from_ratio(swap_amount, target_receive_amount).to_string(),
            "0.000000001336999998"
        );
        assert_eq!(
            calculate_target_price(
                &vault,
                &pair,
                target_receive_amount,
                decimal_delta,
                precision
            )
            .unwrap()
            .to_string(),
            "0.00000000133699"
        );
    }

    #[test]
    fn for_fin_sell_with_decimal_delta_should_truncate() {
        let swap_amount = Uint128::new(747943156999999);
        let target_receive_amount = Uint128::new(1000000);
        let decimal_delta = 12;
        let precision = Precision::DecimalPlaces(2);

        let pair = Pair::default();

        let vault = Vault {
            swap_amount: swap_amount,
            target_denom: pair.quote_denom.clone(),
            balance: Coin::new(100, pair.base_denom.clone()),
            ..Vault::default()
        };

        assert_eq!(
            Decimal::from_ratio(target_receive_amount, swap_amount).to_string(),
            "0.000000001336999998"
        );
        assert_eq!(
            calculate_target_price(
                &vault,
                &pair,
                target_receive_amount,
                decimal_delta,
                precision
            )
            .unwrap()
            .to_string(),
            "0.00000000133699"
        );
    }
}
