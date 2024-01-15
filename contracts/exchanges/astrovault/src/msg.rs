use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw20::Cw20ReceiveMsg;
use exchange_macros::{exchange_execute, exchange_query};
use crate::types::pair::Pair;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub dca_contract_address: Addr,
}

#[cw_serde]
pub struct MigrateMsg {
    pub admin: Addr,
    pub dca_contract_address: Addr,
}

#[cw_serde]
pub enum InternalExecuteMsg {
    CreatePairs { pairs: Vec<Pair> },
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


#[exchange_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}


#[exchange_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
}