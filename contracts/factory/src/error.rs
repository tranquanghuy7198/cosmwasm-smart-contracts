use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FactoryErr {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("FactoryErr: {account:?} is not admin")]
    NotAdmin { account: String },

    #[error("FactoryErr: caller is not operator ({account:?})")]
    NotOperator { account: String },

    #[error("FactoryErr: lengths mismatch")]
    LengthMismatch {},

    #[error("FactoryErr: not setup")]
    ContractNotSetup {},
}
