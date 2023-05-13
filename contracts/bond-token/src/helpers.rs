use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_FEE_PERCENTAGE: u128 = 10000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FunctionSetup {
    pub transfer: bool,
    pub burn: bool,
    pub mint_to_investor: bool,
    pub subscribe: bool,
}

#[cw_serde]
pub struct Holder {
    pub account: String,
    pub balance_in_currency: Uint128,
}

#[cw_serde]
#[derive(Eq)]
pub struct Denomination {
    // X currency_amount = Y bond_amount
    pub currency_amount: Uint128,
    pub bond_amount: Uint128,
}

#[cw_serde]
pub struct BondTokenResponse {}

#[cw_serde]
pub struct HoldersResponse {
    pub holders: Vec<Holder>,
}

#[cw_serde]
pub struct IssuerResponse {
    pub issuer: String,
}

#[cw_serde]
pub struct CurrencyResponse {
    pub currency: String,
}

#[cw_serde]
pub struct RedemptionAmountResponse {
    pub redemption_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Phase {
    Subscription,
    Distribution,
    Coupon,
    Redemption,
}
