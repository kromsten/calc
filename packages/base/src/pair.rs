use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct OldPair {
    pub address: Addr,
    pub base_denom: String,
    pub quote_denom: String,
}
