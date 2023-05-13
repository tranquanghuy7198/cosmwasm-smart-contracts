use bond_token::helpers::{Denomination, FunctionSetup};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMarketingInfo;

use crate::helpers::ContractInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub operators: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    SendAsset {
        request_id: String,
        asset_address: String,
        amount: Uint128,
        recipient: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
