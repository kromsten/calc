use crate::types::{config::Config, pair::Pair};
use cosmwasm_std::Addr;

use super::constants::{ADMIN, DENOM_STAKE, DENOM_UOSMO};

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_denom: DENOM_UOSMO.to_string(),
            quote_denom: DENOM_STAKE.to_string(),
            route: vec![3],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            admin: Addr::unchecked(ADMIN),
        }
    }
}
