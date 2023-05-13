use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlaceholderErr {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("PlaceholderErr: {account:?} is not admin")]
    NotAdmin { account: String },

    #[error("PlaceholderErr: caller is not operator ({account:?})")]
    NotOperator { account: String },

    #[error("PlaceholderErr: lengths mismatch")]
    LengthMismatch {},

    #[error("PlaceholderErr: caller is not bond token ({account:?}")]
    NotBondToken { account: String },

    #[error("PlaceholderErr: caller is not router ({account:?}")]
    NotRouter { account: String },

    #[error("PlaceholderErr: not setup")]
    ContractNotSetup {},
}
