use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Uint64};

#[cw_serde]
pub struct ContractInfo {
    pub code_id: Uint64,
    pub instantiate_msg: Binary,
    pub funds: Vec<Coin>,
    pub label: String,
}
