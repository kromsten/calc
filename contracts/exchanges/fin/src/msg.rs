use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::types::pair::Pair;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub enum InternalMsg {
    CreatePairs { pairs: Vec<Pair> },
}
