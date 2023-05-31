use cosmwasm_std::Addr;

use crate::types::pair::Pair;

pub const ADMIN: &str = "admin";

pub const DENOM_UKUJI: &str = "ukuji";
pub const DENOM_UUSK: &str = "uusk";

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair-address"),
        }
    }
}
