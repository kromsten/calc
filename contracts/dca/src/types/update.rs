use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Update {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}
