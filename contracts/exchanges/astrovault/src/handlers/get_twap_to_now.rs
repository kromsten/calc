use cosmwasm_std::{Coin, Decimal256, Deps, StdError, StdResult};

use super::get_expected_receive_amount::get_expected_receive_amount_handler;

pub const AMOUNT_TO_SIMULATE_TWAP: u128 = 1_000_000u128;

pub fn get_twap_to_now_handler(
    deps: Deps,
    swap_denom: String,
    target_denom: String,
    period: u64,
) -> StdResult<Decimal256> {
    if period != 0 {
        return Err(StdError::generic_err(format!(
            "Cannot get twap for period of {} seconds, only 0 is supported",
            period
        )));
    }

    let coin = get_expected_receive_amount_handler(
        deps,
        Coin {
            denom: swap_denom,
            amount: AMOUNT_TO_SIMULATE_TWAP.into(),
        },
        target_denom,
    )?;

    Ok(Decimal256::from_ratio(
        AMOUNT_TO_SIMULATE_TWAP,
        coin.amount.u128(),
    ))
}

#[cfg(test)]
mod get_twap_to_now_tests {
    use cosmwasm_std::{testing::mock_dependencies, StdError};

    use crate::{
        handlers::get_twap_to_now::get_twap_to_now_handler,
        tests::constants::{DENOM_AARCH, DENOM_UUSDC},
    };

    #[test]
    fn with_period_larger_than_zero_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                DENOM_AARCH.to_string(),
                DENOM_UUSDC.to_string(),
                10
            )
            .unwrap_err(),
            StdError::generic_err("Cannot get twap for period of 10 seconds, only 0 is supported")
        )
    }

    #[test]
    fn with_no_pair_for_denoms_fails() {
        let err = get_twap_to_now_handler(
            mock_dependencies().as_ref(),
            DENOM_AARCH.to_string(),
            DENOM_UUSDC.to_string(),
            0,
        )
        .unwrap_err();

        assert_eq!(err, StdError::generic_err("Pair not found"));
    }
}
