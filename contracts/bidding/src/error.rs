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
    #[error("The provided id doesn't exist.")]
    InvalidId,
}
