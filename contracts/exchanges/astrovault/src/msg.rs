use astrovault::assets::asset::Asset;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg};
use cw20::Cw20ReceiveMsg;
use exchange_macros::{exchange_execute, exchange_query};
use crate::types::pair::{Pair, PopulatedPair};


#[cw_serde]
pub struct InstantiateMsg {
    pub admin:                  Addr,
    pub dca_contract_address:   Addr,
    pub router_address:         Addr,
    pub allow_implicit:         Option<bool>,
}

#[cw_serde]
pub struct InstantiateOptionalMsg {
    pub admin:                  Option<Addr>,
    pub dca_contract_address:   Option<Addr>,
    pub router_address:         Option<Addr>,
    pub allow_implicit:         Option<bool>,
}


pub type MigrateMsg = InstantiateOptionalMsg;


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

    #[returns(Vec<PopulatedPair>)]
    GetPairsFull {
        start_after: Option<Pair>,
        limit: Option<u16>,
    },

    #[returns(CosmosMsg)]
    SwapMsg {
        offer_asset:            Asset,
        minimum_receive_amount: Asset,
        funds:                  Vec<Coin>,
        route:                  Option<Binary>
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