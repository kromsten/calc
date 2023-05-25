use cosmwasm_std::{Decimal, Order, StdResult, Storage};
use cw_storage_plus::Map;

const CUSTOM_FEES: Map<String, Decimal> = Map::new("fees_v20");

pub fn create_custom_fee(
    storage: &mut dyn Storage,
    denom: String,
    swap_fee_percent: Decimal,
) -> StdResult<()> {
    CUSTOM_FEES.save(storage, denom, &swap_fee_percent)
}

pub fn remove_custom_fee(storage: &mut dyn Storage, denom: String) {
    CUSTOM_FEES.remove(storage, denom);
}

pub fn get_custom_fee(storage: &dyn Storage, denom: String) -> StdResult<Option<Decimal>> {
    CUSTOM_FEES.may_load(storage, denom)
}

pub fn get_custom_fees(storage: &dyn Storage) -> StdResult<Vec<(String, Decimal)>> {
    CUSTOM_FEES
        .range(storage, None, None, Order::Ascending)
        .collect()
}
