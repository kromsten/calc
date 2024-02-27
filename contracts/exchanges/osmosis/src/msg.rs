use crate::types::pair::Pair;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};
use exchange_macros::{exchange_execute, exchange_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub struct MigrateMsg {}

#[exchange_execute]
#[cw_serde]
pub enum ExecuteMsg {
    CreatePairs { pairs: Vec<Pair> },
    DeletePairs { pairs: Vec<Pair> },
}

#[exchange_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Pair>)]
    Pairs {
        start_after: Option<Pair>,
        limit: Option<u16>,
    },
}
