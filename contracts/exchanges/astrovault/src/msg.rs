use crate::types::pair::{Pair, PopulatedPair};
use astrovault::assets::asset::Asset;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg};
use exchange_macros::{exchange_execute, exchange_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub dca_contract_address: Addr,
    pub router_address: Addr,
    pub allow_implicit: Option<bool>,
}

#[cw_serde]
pub struct InstantiateOptionalMsg {
    pub admin: Option<Addr>,
    pub dca_contract_address: Option<Addr>,
    pub router_address: Option<Addr>,
    pub allow_implicit: Option<bool>,
}

pub type MigrateMsg = InstantiateOptionalMsg;

#[exchange_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Pair>)]
    Pairs {
        start_after: Option<Pair>,
        limit: Option<u16>,
    },
    #[returns(Vec<PopulatedPair>)]
    PopulatedPairs {
        start_after: Option<Pair>,
        limit: Option<u16>,
    },
    #[returns(CosmosMsg)]
    SwapMsg {
        offer_asset: Asset,
        minimum_receive_amount: Asset,
        funds: Vec<Coin>,
        route: Option<Binary>,
    },
}

#[exchange_execute]
#[cw_serde]
pub enum ExecuteMsg {
    CreatePairs { pairs: Vec<Pair> },
}
