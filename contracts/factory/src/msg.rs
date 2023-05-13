use bond_token::helpers::{Denomination, FunctionSetup};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMarketingInfo;

use crate::helpers::ContractInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub currency_code_id: Uint64,
    pub bond_token_code_id: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        placeholder: String,
        router: String,
    },
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    InstantiateCurrency {
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<Cw20Coin>,
        mint: Option<MinterResponse>,
        marketing: Option<InstantiateMarketingInfo>,
    },
    InstantiateBondToken {
        issuer: String,
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<Cw20Coin>,
        function_setup: FunctionSetup,
        additional_data: String,
        currency: String,
        denomination: Denomination,
        subscription_fee_percentage: Option<Uint128>,
        subscription_fee: Option<Uint128>,
    },
    InstantiateBatch {
        contract_infos: Vec<ContractInfo>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
