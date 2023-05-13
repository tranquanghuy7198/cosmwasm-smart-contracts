use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetVaultErr {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("AssetVaultErr: {account:?} is not admin")]
    NotAdmin { account: String },

    #[error("AssetVaultErr: caller is not operator ({account:?})")]
    NotOperator { account: String },
}
