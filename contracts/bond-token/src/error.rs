use cosmwasm_std::{StdError, Uint128};
use cw20_base::ContractError as BasicError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AdditionalError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("BondToken: function {function:?} not supported")]
    FunctionNotSupported { function: String },

    #[error("BondToken: {caller:?} is not issuer")]
    NotIssuer { caller: String },

    #[error("BondToken: {caller:?} is not router")]
    NotRouter { caller: String },

    #[error("BondToken: fee percentage too high ({percentage:?})")]
    FeePercentageTooHigh { percentage: Uint128 },

    #[error("BondToken: insufficient subscription amount ({amount:?})")]
    InsufficientSubscriptionAmount { amount: Uint128 },

    #[error("BondToken: caller is not router nor placeholder ({caller:?})")]
    NotRouterPlaceholder { caller: String },

    #[error("BondToken: {action:?} is not allowed in this phase")]
    ActionNotAllowed { action: String },

    #[error("BondToken: invalid phase")]
    InvalidPhase,
}

#[derive(Error, Debug, PartialEq)]
pub enum BondTokenErr {
    Std(#[from] StdError),
    BasicError(BasicError),
    AdditionalError(AdditionalError),
}

impl fmt::Display for BondTokenErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
