use crate::types::pair::Pair;

use super::constants::{DENOM_STAKE, DENOM_UOSMO};

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_denom: DENOM_UOSMO.to_string(),
            quote_denom: DENOM_STAKE.to_string(),
            route: vec![3],
        }
    }
}
