use crate::{
    error::ContractError, helpers::validation_helpers::assert_sender_is_executor,
    state::swap_adjustments::update_swap_adjustments,
};
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response};
use fin_helpers::position_type::OldPositionType;

pub fn update_swap_adjustments_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    position_type: OldPositionType,
    adjustments: Vec<(u8, Decimal)>,
) -> Result<Response, ContractError> {
    assert_sender_is_executor(deps.storage, &env, &info.sender)?;
    update_swap_adjustments(deps.storage, position_type, adjustments, env.block.time)?;
    Ok(Response::new())
}
