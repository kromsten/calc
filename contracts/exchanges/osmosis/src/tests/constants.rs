use cosmwasm_std::{Decimal, Uint128};

pub const ONE: Uint128 = Uint128::new(1000000);
pub const TEN: Uint128 = Uint128::new(10000000);

pub const ONE_DECIMAL: Decimal = Decimal::new(Uint128::new(1000000000000000000));

pub const SWAP_FEE_RATE: &str = "0.001";

pub const USER: &str = "user";
pub const ADMIN: &str = "admin";
pub const DCA_CONTRACT_ADDRESS: &str = "dca_contract_address";
pub const LIMIT_ORDER_ADDRESS: &str = "limit_order_address";

pub const DENOM_UOSMO: &str = "uosmo";
pub const DENOM_UATOM: &str = "uatom";
pub const DENOM_UION: &str = "uion";
pub const DENOM_USDC: &str = "usdc";
pub const DENOM_STAKE: &str = "ustake";
