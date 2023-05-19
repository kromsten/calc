use cosmwasm_std::{Decimal, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use fin_helpers::position_type::OldPositionType;

const BUY_ADJUSTMENTS: Map<u8, Decimal> = Map::new("buy_adjustments_v20");
const SELL_ADJUSTMENTS: Map<u8, Decimal> = Map::new("sell_adjustments_v20");
const BUY_ADJUSTMENTS_UPDATED_AT: Item<Timestamp> = Item::new("buy_adjustments_updated_at_v20");
const SELL_ADJUSTMENTS_UPDATED_AT: Item<Timestamp> = Item::new("buy_adjustments_updated_at_v20");

fn last_updated(storage: &dyn Storage, position_type: OldPositionType) -> StdResult<Timestamp> {
    match position_type {
        OldPositionType::Enter => BUY_ADJUSTMENTS_UPDATED_AT.load(storage),
        OldPositionType::Exit => SELL_ADJUSTMENTS_UPDATED_AT.load(storage),
    }
}

fn adjustments_updated_store(position_type: OldPositionType) -> &'static Item<'static, Timestamp> {
    match position_type {
        OldPositionType::Enter => &BUY_ADJUSTMENTS_UPDATED_AT,
        OldPositionType::Exit => &SELL_ADJUSTMENTS_UPDATED_AT,
    }
}

pub fn adjustments_store(position_type: OldPositionType) -> &'static Map<'static, u8, Decimal> {
    match position_type {
        OldPositionType::Enter => &BUY_ADJUSTMENTS,
        OldPositionType::Exit => &SELL_ADJUSTMENTS,
    }
}

pub fn update_swap_adjustments(
    storage: &mut dyn Storage,
    position_type: OldPositionType,
    adjustments: Vec<(u8, Decimal)>,
    block_time: Timestamp,
) -> StdResult<()> {
    for (model, adjustment) in adjustments {
        adjustments_store(position_type.clone()).save(storage, model, &adjustment)?;
    }
    adjustments_updated_store(position_type).save(storage, &block_time)
}

pub fn get_swap_adjustment(
    storage: &dyn Storage,
    position_type: OldPositionType,
    model: u8,
    block_time: Timestamp,
) -> StdResult<Decimal> {
    let last_updated = last_updated(storage, position_type.clone())?;
    let thirty_hours = 30 * 60 * 60;
    if last_updated < block_time.minus_seconds(thirty_hours) {
        return Ok(Decimal::one());
    }
    adjustments_store(position_type).load(storage, model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Decimal,
    };
    use fin_helpers::position_type::OldPositionType;

    #[test]
    fn gets_swap_adjustment_if_updated_within_30_hours() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_swap_adjustments(
            deps.as_mut().storage,
            OldPositionType::Enter,
            vec![(30, Decimal::percent(90))],
            env.block.time,
        )
        .unwrap();

        let adjustment = get_swap_adjustment(
            deps.as_ref().storage,
            OldPositionType::Enter,
            30,
            env.block.time,
        )
        .unwrap();

        assert_eq!(adjustment, Decimal::percent(90));
    }

    #[test]
    fn gets_default_swap_adjustment_if_not_updated_within_30_hours() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        update_swap_adjustments(
            deps.as_mut().storage,
            OldPositionType::Enter,
            vec![(30, Decimal::percent(90))],
            env.block.time,
        )
        .unwrap();

        let adjustment = get_swap_adjustment(
            deps.as_ref().storage,
            OldPositionType::Enter,
            30,
            env.block.time.plus_seconds(31 * 60 * 60),
        )
        .unwrap();

        assert_eq!(adjustment, Decimal::one());
    }
}
