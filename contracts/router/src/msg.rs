use std::vec;

use cosmwasm_schema::{cw_serde, QueryResponses};
use placeholder::helpers::InvesmentRule;

use crate::helpers::{
    Coupon, Cw20BatchBalanceQuery, Cw20BatchBalanceResponse, Cw20MintItem, Cw20TransferItem,
};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        placeholder: String,
        factory: String,
    },
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    Cw20MintBatch {
        cw20_mint_items: Vec<Cw20MintItem>,
    },
    Cw20TransferBatch {
        cw20_transfer_items: Vec<Cw20TransferItem>,
    },
    Distribute {
        bond_token: String,
        investment_rules: Vec<InvesmentRule>,
    },
    SendCoupon {
        bond_token: String,
        coupons: Vec<Coupon>,
    },
    Redeem {
        bond_token: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Cw20BatchBalanceResponse)]
    Cw20QueryBalanceBatch {
        cw20_batch_balance_queries: Vec<Cw20BatchBalanceQuery>,
    },
}
