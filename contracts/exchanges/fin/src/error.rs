use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid funds: {msg:?}")]
    InvalidFunds { msg: String },

    #[error("Failed swap: {msg:?}")]
    FailedSwap { msg: String },

    #[error("Missing reply id")]
    MissingReplyId {},
}
