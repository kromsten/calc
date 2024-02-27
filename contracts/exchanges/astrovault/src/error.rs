use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Failed swap: {msg:?}")]
    FailedSwap { msg: String },

    #[error("Missing reply id")]
    MissingReplyId {},

    #[error("Invalid pair info: {msg:?}")]
    InvalidPair { msg: String },

    #[error("There is no route for the give pair")]
    NoRoutedPair {},

    #[error("Invalid route. Can't get from: {base:?} to {quote:?}")]
    InvalidRoute { base: String, quote: String },

    #[error("Assets cannot be the same")]
    SameAsset {},

    #[error("Assets cannot be empty")]
    EmptyAsset {},

    #[error("Couldn't get asset info from the pool")]
    AssetQueryFail {},

    #[error("Error reconstructing a route. Invalid Hop info")]
    InvalidHops {},

    #[error("Last hop in a route must have info about next connecting pool ")]
    MissingNextPoolHop {},

    #[error("Pair not found")]
    PairNotFound {},

    #[error("Route not found")]
    RouteNotFound {},

    #[error("Route is empty")]
    RouteEmpty {},

    #[error("Route assets are not unique and contains duplicates")]
    RouteDuplicates {},

    #[error("Pool not found")]
    PoolNotFound {},

    #[error("Error getting assets info from the pool")]
    PoolError {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),
}
