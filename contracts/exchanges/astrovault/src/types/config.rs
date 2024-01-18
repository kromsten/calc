use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub dca_contract_address: Addr,
    pub router_address: Addr,
}

pub type RouterConfig = astrovault::router::query_msg::ConfigResponse;