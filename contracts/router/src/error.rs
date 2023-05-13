use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterErr {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("RouterErr: {account:?} is not admin")]
    NotAdmin { account: String },

    #[error("RouterErr: caller is not operator ({account:?})")]
    NotOperator { account: String },

    #[error("RouterErr: lengths mismatch")]
    LengthMismatch {},

    #[error("RouterErr: not setup")]
    ContractNotSetup {},

    #[error("RouterErr: bond token not registered ({bond_token:?})")]
    InvalidBondToken { bond_token: String },

    #[error("RouterErr: caller is not issuer of bond token ({caller:?} {bond_token:?})")]
    NotIssuer { caller: String, bond_token: String },
}
