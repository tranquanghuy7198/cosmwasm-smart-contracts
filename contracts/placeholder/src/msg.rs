use crate::helpers::{BondValidationResponse, InvesmentRule, SubscriptionsResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        factory: String,
        router: String,
    },
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    // RegisterCurrencies {
    //     currencies: Vec<String>,
    //     states: Vec<bool>,
    // },
    RegisterBondToken {
        bond_token: String,
    },
    RegisterSubscription {
        investor: String,
        currency: String,
        subscription_amount: Uint128,
        fee_amount: Uint128,
    },
    ReleaseCurrency {
        issuer: String,
        bond_token: String,
        currency: String,
        investment_rules: Vec<InvesmentRule>,
    },
    WithdrawSystemFee {
        recipient: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    #[returns(SubscriptionsResponse)]
    SubscriptionsOf { bond_token: String },

    #[returns(BondValidationResponse)]
    ValidateBondToken { bond_token: String },
}
