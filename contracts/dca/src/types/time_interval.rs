use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum TimeInterval {
    EveryBlock,
    EveryMinute,
    HalfHourly,
    Hourly,
    HalfDaily,
    Daily,
    Weekly,
    Fortnightly,
    Monthly,
    Custom { seconds: u64 },
}
