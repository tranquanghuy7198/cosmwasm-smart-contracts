use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const NATIVE_DENOM: &str = "flavor";

// Which investor has subscribed how much currency
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Subscription {
    pub investor: Addr,
    pub currency_amount: Uint128,
}

// Which investor is allowed to subscribe maximum how much currency
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InvesmentRule {
    pub investor: String,
    pub currency_amount: Uint128,
}

#[cw_serde]
pub struct SubscriptionsResponse {
    pub subscriptions: Vec<SubscriptionResponse>,
}

#[cw_serde]
pub struct BondValidationResponse {
    pub validity: bool,
}

#[cw_serde]
pub struct SubscriptionResponse {
    pub investor: String,
    pub currency_amount: Uint128,
}
