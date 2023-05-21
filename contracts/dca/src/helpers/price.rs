use crate::types::{pair::Pair, position_type::PositionType};
use cosmwasm_std::{Coin, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
use kujira::{
    asset::{Asset, AssetInfo},
    denom::Denom,
    fin::QueryMsg,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinPoolResponse {
    pub quote_price: Decimal,
    pub total_offer_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinBookResponse {
    pub base: Vec<FinPoolResponse>,
    pub quote: Vec<FinPoolResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

pub fn calculate_slippage(actual_price: Decimal, belief_price: Decimal) -> Decimal {
    let difference = actual_price
        .checked_sub(belief_price)
        .unwrap_or(Decimal::zero());

    if difference.is_zero() {
        return Decimal::zero();
    }

    difference / belief_price
}
