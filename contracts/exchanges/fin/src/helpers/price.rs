use std::str::FromStr;

use cosmwasm_std::{Decimal256, StdError, StdResult};
use kujira_std::{Precise, Precision};

use crate::types::{pair::Pair, position_type::PositionType};

pub fn get_fin_price(
    target_price: Decimal256,
    swap_denom: String,
    pair: &Pair,
) -> StdResult<Decimal256> {
    if pair.decimal_delta < 0 {
        return Err(StdError::GenericErr {
            msg: "Negative decimal deltas are not supported".to_string(),
        });
    }

    let directional_price = match pair.position_type(&swap_denom) {
        PositionType::Enter => target_price,
        PositionType::Exit => Decimal256::one() / target_price,
    };

    if pair.decimal_delta == 0 {
        return Ok(directional_price.round(&Precision::DecimalPlaces(pair.price_precision)));
    }

    let adjustment = Decimal256::from_str(
        &10u128
            .pow(pair.decimal_delta.unsigned_abs() as u32)
            .to_string(),
    )
    .unwrap();

    let rounded_price = directional_price
        .checked_mul(adjustment)
        .unwrap()
        .round(&Precision::DecimalPlaces(pair.price_precision));

    Ok(rounded_price.checked_div(adjustment).unwrap())
}

#[cfg(test)]
mod calculate_target_price_tests {
    use cosmwasm_std::{
        testing::mock_dependencies, to_binary, Addr, ContractResult, Decimal256, SystemResult,
        Uint128, Uint256,
    };
    use cw20::Denom;
    use kujira_fin::ConfigResponse;
    use kujira_std::Precision;

    use crate::tests::constants::{DENOM_UKUJI, DENOM_UUSK};

    use super::*;

    #[test]
    fn should_be_correct_when_buying_on_fin() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(3),
                    decimal_delta: 0,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::percent(500);

        assert_eq!(
            get_fin_price(target_price, pair.quote_denom.clone(), &pair).unwrap(),
            target_price
        );
    }

    #[test]
    fn should_be_correct_when_selling_on_fin() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(3),
                    decimal_delta: 0,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::percent(500);

        assert_eq!(
            get_fin_price(target_price, pair.base_denom.clone(), &pair).unwrap(),
            Decimal256::one() / target_price
        );
    }

    #[test]
    fn should_truncate_price_to_pair_precision_decimal_places() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(3),
                    decimal_delta: 0,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::percent(300);

        assert_eq!(
            get_fin_price(target_price, pair.base_denom.clone(), &pair).unwrap(),
            (Decimal256::one() / target_price).round(&Precision::DecimalPlaces(3))
        );
    }

    #[test]
    fn for_fin_buy_should_truncate_price_to_pair_precision_plus_decimal_delta_decimal_places() {
        let mut deps = mock_dependencies();

        let pair = Pair {
            price_precision: 2,
            decimal_delta: 12,
            ..Pair::default()
        };

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(2),
                    decimal_delta: 12,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price = Decimal256::from_ratio(
            Uint256::from_u128(1000000),
            Uint256::from_u128(747943156999999),
        );

        assert_eq!(
            get_fin_price(target_price, pair.quote_denom.clone(), &pair).unwrap(),
            target_price.round(&Precision::DecimalPlaces(12 + 2))
        );
    }

    #[test]
    fn for_fin_sell_should_truncate_price_to_pair_precision_plus_decimal_delta_decimal_places() {
        let mut deps = mock_dependencies();

        let pair = Pair {
            price_precision: 2,
            decimal_delta: 12,
            ..Pair::default()
        };

        deps.querier.update_wasm(move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    price_precision: Precision::DecimalPlaces(2),
                    decimal_delta: 12,
                    owner: Addr::unchecked("Hans"),
                    denoms: [
                        Denom::Native(DENOM_UKUJI.to_string()),
                        Denom::Native(DENOM_UUSK.to_string()),
                    ],
                    is_bootstrapping: false,
                    fee_taker: Decimal256::one(),
                    fee_maker: Decimal256::one(),
                })
                .unwrap(),
            ))
        });

        let target_price =
            Decimal256::from_ratio(Uint128::new(1000000), Uint128::new(747943156999999));

        assert_eq!(
            get_fin_price(target_price, pair.base_denom.clone(), &pair).unwrap(),
            (Decimal256::one() / target_price).round(&Precision::DecimalPlaces(12 + 2))
        );
    }
}
