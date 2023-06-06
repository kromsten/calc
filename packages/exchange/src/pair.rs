use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Pair {
    pub denoms: [String; 2],
}
