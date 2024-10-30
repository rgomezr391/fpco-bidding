use cosmwasm_std::{Addr, StdError};
use cw_utils::PaymentError;

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{sender} is not contract admin")]
    Unauthorized { sender: Addr },
    #[error("Payment error: {0}")]
    Payment(#[from] PaymentError),
    #[error("The provided auction id doesn't exist.")]
    InvalidAuctionId,
    #[error("The provided bid item id doesn't exist.")]
    InvalidBidItemId,
    #[error("The auction is in a non-updateable state.")]
    AuctionNonUpdateable,
    #[error("The auction is already completed and can't accept bids.")]
    AuctionCompleted,
    #[error("{msg}")]
    AuctionInvalidStateUpdate { msg: String },
    #[error("Expecting to receive {denom}.")]
    NoFundsReceived{ denom: String },
    #[error("{msg}.")]
    UnexpectedAssetsReceived{ msg: String },
}
