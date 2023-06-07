use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};

#[cw_serde]
pub enum TriggerConfiguration {
    Time {
        target_time: Timestamp,
    },
    Price {
        target_price: Decimal,
        order_idx: Uint128,
    },
}

#[cw_serde]
pub struct Trigger {
    pub vault_id: Uint128,
    pub configuration: TriggerConfiguration,
}
