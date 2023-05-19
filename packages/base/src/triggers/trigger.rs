use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Timestamp, Uint128};
use enum_as_inner::EnumAsInner;

#[cw_serde]
pub enum OldTimeInterval {
    EverySecond,
    EveryMinute,
    HalfHourly,
    Hourly,
    HalfDaily,
    Daily,
    Weekly,
    Fortnightly,
    Monthly,
}

#[derive(EnumAsInner)]
#[cw_serde]
pub enum OldTriggerConfiguration {
    Time {
        target_time: Timestamp,
    },
    FinLimitOrder {
        target_price: Decimal256,
        order_idx: Option<Uint128>,
    },
}

#[cw_serde]
pub struct OldTrigger {
    pub vault_id: Uint128,
    pub configuration: OldTriggerConfiguration,
}
