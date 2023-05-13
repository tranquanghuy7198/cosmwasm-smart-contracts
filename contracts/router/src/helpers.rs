use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const NATIVE_DENOM: &str = "flavor";

// How much currency which investor receives
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Coupon {
    pub investor: String,
    pub currency_amount: Uint128,
}

#[cw_serde]
pub struct Cw20MintItem {
    pub cw20_token: String,
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw20TransferItem {
    pub cw20_token: String,
    pub sender: String,
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw20BatchBalanceQuery {
    pub cw20_token: String,
    pub cw20_holder: String,
}

#[cw_serde]
pub struct Cw20BatchBalanceResponse {
    pub balances: Vec<Uint128>,
}
