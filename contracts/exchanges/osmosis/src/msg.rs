use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::types::pair::Pair;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum InternalExternalMsg {
    CreatePairs { pairs: Vec<Pair> },
    DeletePairs { pairs: Vec<Pair> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum InternalQueryMsg {
    #[returns(Vec<Pair>)]
    GetPairs {
        start_after: Option<Pair>,
        limit: Option<u16>,
    },
}
