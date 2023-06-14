use crate::types::pair::Pair;

use super::constants::{DENOM_UATOM, DENOM_UOSMO};

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_denom: DENOM_UOSMO.to_string(),
            quote_denom: DENOM_UATOM.to_string(),
            route: vec![1, 2, 3],
            decimal_delta: 0,
            price_precision: 3,
        }
    }
}
